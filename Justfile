run rom:
    cargo run --no-default-features --features=sdl -- '{{rom}}'

serve:
    cd web && npm install && npx webpack serve

test:
    cargo test