docker run \
  --rm \
  -v $(pwd):/root/src \
  -v /tmp/registry:/usr/local/cargo/registry \
  -w /root/src \
  notfl3/cargo-apk /bin/bash -c "rustup target add armv7-linux-androideabi && cargo quad-apk build --release --package mqt"