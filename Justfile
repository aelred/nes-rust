# Run the given ROM file
run rom:
    cargo run --no-default-features --features=sdl -- {{quote(rom)}}

profile rom:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo run --release \
      --no-default-features --features=sdl -- {{quote(rom)}}

# Build the web version, storing the result in `./web`
build-web:
    #!/usr/bin/env bash
    set -e
    tmpdir=$(mktemp -d)

    # Build WASM
    # Various flags needed because we want to use threads, atomics and shared memory in WASM
    cargo build --bin nes-rust --release --target wasm32-unknown-unknown \
      --no-default-features --features=web \
      -Zbuild-std=std,panic_abort \
      --config "target.wasm32-unknown-unknown.rustflags='\
        -Ctarget-feature=+atomics -Clink-args=--shared-memory -Clink-args=--max-memory=1073741824 \
        -Clink-args=--import-memory -Clink-args=--export=__wasm_init_tls -Clink-args=--export=__tls_size \
        -Clink-args=--export=__tls_align -Clink-args=--export=__tls_base'"

    # Generate JS glue that lets WASM interact with web APIs
    wasm-bindgen target/wasm32-unknown-unknown/release/nes-rust.wasm --target web --out-dir "$tmpdir" --no-typescript

    # Optimise WASM
    wasm-opt "$tmpdir/nes-rust_bg.wasm" --output "$tmpdir/nes-rust_bg.wasm" -O4 --debuginfo --dce

    cp -a "$tmpdir/." ./web

# Monitor for changes and rebuild the web version
watch-web:
    cargo watch --shell 'just build-web' --ignore ./web

# Host web server
run-server:
    vite ./web --open

# Build and serve the web version with hot reload
[parallel]
serve: watch-web run-server

# Run all tests
test:
    cargo test

# Provision infrastructure using Terraform
provision:
    cd deploy/infrastructure && terraform apply