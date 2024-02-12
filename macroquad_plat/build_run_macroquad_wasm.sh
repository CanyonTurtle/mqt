cargo build --release --target wasm32-unknown-unknown \
    && install target/wasm32-unknown-unknown/release/mqt.wasm cart.wasm \
    && basic-http-server .