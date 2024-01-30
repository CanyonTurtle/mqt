cargo build --release --target x86_64-pc-windows-gnu \
    && install target/x86_64-pc-windows-gnu/release/mqt.exe mqt.exe \
    && ./mqt.exe