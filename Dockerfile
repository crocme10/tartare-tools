FROM ekidd/rust-musl-builder as builder
COPY osm-utils osm-utils
COPY Cargo.toml .
COPY src src
RUN ["cargo", "build" ,"--release"]

FROM scratch
WORKDIR /bin
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/extract-osm-pois .
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/improve-stop-positions .
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/merge-pois .
VOLUME /app/input
VOLUME /app/output
