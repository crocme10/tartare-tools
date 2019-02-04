FROM ekidd/rust-musl-builder as builder
COPY osm-utils osm-utils
COPY navitia-poi-model navitia-poi-model
COPY Cargo.toml .
COPY src src
RUN ["cargo", "build" ,"--release"]

FROM scratch
WORKDIR /bin
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/extract-osm-pois .
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/improve-stop-positions .
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/merge-pois .
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/sytral2navitia-pois .
VOLUME /app/input
VOLUME /app/output
