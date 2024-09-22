# Run the given ROM file
run rom:
    cargo run --no-default-features --features=sdl -- '{{rom}}'

# Build the web version, storing the result in `./web/dist`
build-web:
    cd web && npm install && npx webpack

# Build and serve the web version
serve:
    cd web && npm install && npx webpack serve

# Run all tests
test:
    cargo test

# Provision infrastructure using Terraform
provision:
    cd deploy/infrastructure && terraform apply