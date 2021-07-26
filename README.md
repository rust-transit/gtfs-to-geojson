# GTFS to GeoJson

This Rust crate is used to convert GTFS data to the GeoJSON format.

## How to compile and run the standalone program

* Clone this repository
* [Install Rust](https://www.rust-lang.org/tools/install)
* Run the tests with `cargo test --all-features` (see [GitHub actions setup](https://github.com/rust-transit/gtfs-to-geojson/tree/main/.github/workflows))
* Build the optimized binary with `cargo build --release`
* Run the standalone program with `target/release/gtfs-geojson --help`
