set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly-wasm32-unknown-unknown -y
source $HOME/.cargo/env
rustup show
rustc -vV
cargo install trunk
yarn install
trunk build --release
