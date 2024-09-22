run rom:
    cargo run --no-default-features --features=sdl -- '{{rom}}'

build-web:
    cd web && npm install && npx webpack

serve:
    cd web && npm install && npx webpack serve

test:
    cargo test