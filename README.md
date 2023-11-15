# GTFS to GeoJson

This Rust crate is used to convert GTFS data to the GeoJSON format.

## How to compile and run the standalone program

* Clone this repository
* [Install Rust](https://www.rust-lang.org/tools/install)
* Run the tests with `cargo test --all-features` (see [GitHub actions setup](https://github.com/rust-transit/gtfs-to-geojson/tree/main/.github/workflows))
* Build the optimized binary with `cargo build --release`
* Run the standalone program with `cargo run --release -- --help`
* To run the standalone program without cargo (e.g. when shipping the binary), run `target/release/gtfs-geojson --help`

## Using Docker

```bash
docker build -t gtfs-to-geojson .
```

Check the build with:

```bash
docker run gtfs-to-geojson --help
```

To run with data stored locally:

```bash
docker run -v /path/to/gtfs/datasets/:/data/ gtfs-to-geojson --input /data/gtfs.zip --output /data/output.geojson
```
