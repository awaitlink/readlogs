set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
rustup show
rustc -vV
cargo install trunk
yarn install
trunk build --release
