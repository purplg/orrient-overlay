set dotenv-load := true

link:
    just -f crates/orrient_link/justfile build
    ./link.sh
