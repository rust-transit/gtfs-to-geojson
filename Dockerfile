FROM rust:1.67

COPY . .

RUN cargo build --release

ENTRYPOINT ["target/release/gtfs-geojson"]