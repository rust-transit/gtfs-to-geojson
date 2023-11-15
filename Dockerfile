FROM rust:1.67

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly

COPY . .

RUN cargo +nightly build --release -Z sparse-registry

ENTRYPOINT ["target/release/gtfs-geojson"]