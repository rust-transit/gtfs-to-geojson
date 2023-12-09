# GTFS to GeoJson

This Rust crate is used to convert GTFS data to the GeoJSON format. 

## Installing Crate

[Crate](https://crates.io/crates/gtfs-geojson)

While the repo is named `gtfs-to-geojson` the crate is called `gtfs_geojson` and can me installed using 

```bash
cargo add gtfs_geojson
```

## Functions

The `gtfs_geojson` library use the `gtfs_structures` library and there types to convert gtfs files over to geojson.

### extract_stops(&gtfs: &gtfs_structures::Gtfs) -> Vec<Feature>

`extract_stops` will export all of the stops and their properties from the gtfs.


### extract_trips_shapes(&gtfs: &gtfs_structures::Gtfs) -> Vec<Feature>

`extract_trips_shapes` will export all of the shapes as line features from the shapes.txt file and add in trip information for each feature in the geojson properties from trips.txt.


## How to compile and run the standalone program

* Clone this repository
* [Install Rust](https://www.rust-lang.org/tools/install)
* Run the tests with `cargo test --all-features` (see [GitHub actions setup](https://github.com/rust-transit/gtfs-to-geojson/tree/main/.github/workflows))
* Build the optimized binary with `cargo build --release`
* Run the standalone program with `cargo run --release -- --help`
* To run the standalone program without cargo (e.g. when shipping the binary), run `target/release/gtfs-geojson --help`
