set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/thedodd/trunk --rev 8852981a2982827d45a4a31ce362fae69311b633 trunk
yarn install
trunk build --release
