default:
    just --list

test:
    cargo test

install:
    cargo install --path .
