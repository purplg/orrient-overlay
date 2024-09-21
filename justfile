set dotenv-load := true

watch command:
    cargo watch -d 1 --ignore assets/ --ignore orrient_shim/ --clear -- cargo {{ command }}

run:
    @just watch lrun

build:
    @just watch lbuild

release:
    cargo lrun --release

link:
    just -f crates/orrient_shim/justfile build
    ./link.sh
