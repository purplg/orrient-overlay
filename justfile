set dotenv-load := true

watch *command:
    cargo watch --ignore assets/ --ignore orrient_shim/ --clear -- cargo {{ command }}

run:
    @just watch lrun

build:
    @just watch lbuild

release:
    cargo lrun --release

test:
    cargo test --no-run
    cargo pretty-test --workspace

link:
    just -f crates/orrient_shim/justfile build
    ./link.sh
