//! This crates aims to be a simple converter for GTFS to GeoJSON formats.

use clap::Parser;
use gtfs_structures::GtfsReader;
use std::path::PathBuf;

mod converter;
mod utility;

#[derive(Parser, Debug)]
#[clap(name = "basic")]
struct Opt {
    // GTFS files
    #[clap(
        name = "gtfs",
        short = 'i',
        long = "input",
        help = "Path to the GTFS file (can be a directory or a zip file) or URL to an online GTFS file",
        parse(from_os_str)
    )]
    file: PathBuf,
    #[clap(
        name = "output",
        short = 'o',
        long = "output",
        help = "Path to the output file. If not present, geojson file is outputed in stdout",
        parse(from_os_str)
    )]
    output_file: Option<PathBuf>,
}

fn main() {
    let opt = Opt::parse();

    println!("Reading GTFS");
    let gtfs = GtfsReader::default()
        .read_stop_times(true)
        .read(
            opt.file
                .to_str()
                .expect("Invalid file path. Could not convert to string."),
        )
        .expect("The GTFS file is not well formated.");

    println!("Extracting Spatial features");
    let stops_as_features = crate::converter::convert_to_geojson(&gtfs);

    println!("Saving GeoJSON");
    match opt.output_file {
        Some(f) => utility::save_to_file(&stops_as_features, &f),
        None => println!("{}", stops_as_features),
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    #[test]
    fn with_code_test() {
        use crate::converter::convert_to_geojson;
        let gtfs = gtfs_structures::GtfsReader::default().read_stop_times(true).read("test/basic/gtfs/").unwrap();
        let geojson = convert_to_geojson(&gtfs);

        let given_feature = &geojson.features.into_iter().find(|f| {
            f.properties
                .as_ref()
                .unwrap()
                .get("id")
                .and_then(|id| id.as_str())
                == Some("stop2")
        });

        assert_eq!(
            json!(given_feature.as_ref().unwrap().properties),
            json!({
            "code": "0001",
            "description": "",
            "id": "stop2",
            "name": "StopPoint",
            "wheelchair_boarding": "unknown"

            })
        );

        // long and lat
        assert_eq!(
            json!(given_feature.as_ref().unwrap().geometry),
            json!({
                    "coordinates":[1.0, 47.0],
                    "type":"Point"
                    }
            )
        );
    }

    #[test]
    fn no_code_test() {
        use super::converter::convert_to_geojson;
        let gtfs = gtfs_structures::Gtfs::new("test/basic/gtfs/").unwrap();
        let geojson = convert_to_geojson(&gtfs);

        let given_feature = &geojson.features.into_iter().find(|f| {
            f.properties
                .as_ref()
                .unwrap()
                .get("id")
                .and_then(|id| id.as_str())
                == Some("stop1")
        });

        assert_eq!(
            json!(given_feature.as_ref().unwrap().properties),
            json!({
                "description": "",
                "id": "stop1",
                "name": "Stop Area",
                "wheelchair_boarding": "unknown"
            })
        );

        assert_eq!(
            json!(given_feature.as_ref().unwrap().geometry),
            json!({
                    "coordinates":[0.0, 48.0],
                    "type":"Point"
                }
            )
        );
    }

    #[test]
    fn shape_test() {
        use super::converter::convert_to_geojson;
        let gtfs = gtfs_structures::Gtfs::new("test/basic/gtfs/").unwrap();
        let geojson = convert_to_geojson(&gtfs);

        let given_feature = &geojson.features.into_iter().find(|f| {
            f.properties
                .as_ref()
                .unwrap()
                .get("route_id")
                .and_then(|id| id.as_str())
                == Some("route1")
        });

        assert_eq!(
            json!(given_feature.as_ref().unwrap().properties),
            json!({
                "route_color": "rgb(0,0,0)",
                "route_text_color": "rgb(255,255,255)",
                "route_id": "route1",
                "route_long_name": "100",
                "route_short_name": "100"
            })
        );

        assert_eq!(
            json!(given_feature.as_ref().unwrap().geometry),
            json!({
                    "coordinates":[[0.0,48.0], [1.0,47.0], [1.0,45.0], [2.0,44.0]],
                    "type":"LineString"
                }
            )
        );
    }

    #[test]
    fn empty_shapes_file_generates_line_geometries() {
        use super::converter::convert_to_geojson;
        let gtfs = gtfs_structures::Gtfs::new("test/empty_shapes/gtfs/").unwrap();
        let geojson = convert_to_geojson(&gtfs);

        let given_feature = &geojson.features.into_iter().find(|f| {
            f.properties
                .as_ref()
                .unwrap()
                .get("route_id")
                .and_then(|id| id.as_str())
                == Some("route1")
        });

        assert_eq!(
            json!(given_feature.as_ref().unwrap().geometry),
            json!({
                    "coordinates":[[1.0,47.0], [1.0,45.0]],
                    "type":"LineString"
                }
            )
        );
    }
}
