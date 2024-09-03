set dotenv-load := true

watch:
    cargo watch -d 1 --ignore assets/ --ignore orrient_link/ --clear -- cargo lrun

release:
    cargo lrun --release

link:
    just -f crates/orrient_link/justfile build
    ./link.sh
