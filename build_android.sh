docker run \
  --rm \
  -v $(pwd):/root/src \
  -v /tmp/registry\":/usr/local/cargo/registry\" \
  -w /root/src \
  notfl3/cargo-apk cargo quad-apk build --release