#!/bin/bash

rustup update
rustup component add clippy rustfmt
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-expand

echo "Setup complete!"
