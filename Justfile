# Run the given ROM file
run rom:
    cargo run --no-default-features --features=sdl -- {{quote(rom)}}

profile rom:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo run --release \
      --no-default-features --features=sdl -- {{quote(rom)}}

# Build the web version, storing the result in `./web`
build-web:
    # Various flags needed because we want to use threads, atomics and shared memory in WASM
    rustup run nightly wasm-pack build \
        --target web \
        --out-dir ./web . \
        --no-pack --no-typescript \
        -Zbuild-std=std,panic_abort \
        --config "target.wasm32-unknown-unknown.rustflags='-Ctarget-feature=+atomics -Clink-args=--shared-memory -Clink-args=--max-memory=1073741824 -Clink-args=--import-memory -Clink-args=--export=__wasm_init_tls -Clink-args=--export=__tls_size -Clink-args=--export=__tls_align -Clink-args=--export=__tls_base'" \
        --no-default-features --features=web

# Build and serve the web version
serve:
    cargo watch --shell 'just build-web' --ignore ./web & npx vite

# Run all tests
test:
    cargo test

# Provision infrastructure using Terraform
provision:
    cd deploy/infrastructure && terraform apply