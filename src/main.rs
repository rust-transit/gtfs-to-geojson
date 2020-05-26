//! This crates aims to be a simple converter for GTFS to GeoJSON formats.

use gtfs_structures::Gtfs;
use serde_json::{json};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // GTFS files
    #[structopt(
        name = "gtfs",
        short = "i",
        long = "input",
        help = "Path to the gtfs file. Can be a directory or a zip file",
        parse(from_os_str)
    )]
    file: PathBuf,

    // Output file
    #[structopt(
        name = "json",
        short = "o",
        long = "output",
        help = "Filename of the outputed json data. Doesn't need to exist beforehand. All existing data will be removed.",
        parse(from_os_str)
    )]
    output: PathBuf,

    // To be verbose about what's going on.
    #[structopt(name = "verbose", short = "v", long = "verbose")]
    verbose: bool,
}

pub mod converter {
    use geojson::Value::Point;
    use geojson::{Feature, FeatureCollection};
    use gtfs_structures::Gtfs;
    use serde_json::Map;

    /// This function will take a GTFS data format and ouput a FeatureCollection, which can in turn, be printed by the utility module.
    /// If the verbose argument if True, then it will also print each step of conversion.
    /// # Examples
    /// ```
    /// let gtfs_data = Gtfs::new("tests/gtfs/gtfs_46.zip");
    /// convert_to_geojson(gtfs_data, true);
    /// ```
    pub fn convert_to_geojson(gtfs_data: &Gtfs, verbose: bool) -> FeatureCollection {
        // Convert the stops of the GTFS by mapping each field
        let features = gtfs_data
            .stops
            .values()
            .map(|stop| {
                if verbose {
                    println!("Stop {:?} - {:?} - {:?}", stop.name, stop.id, stop.code);
                    println!("Description {:?}", stop.description);
                }

                let info = vec![
                    ("name", Some(stop.name.clone().into())),
                    ("id", Some(stop.id.clone().into())),
                    ("description", Some(stop.description.clone().into())),
                    ("code", stop.code.as_ref().map(|code| code.clone().into())),
                    (
                        "parent_station",
                        stop.parent_station
                            .as_ref()
                            .map(|parent| parent.clone().into()),
                    ),
                    (
                        "timezone",
                        stop.timezone.as_ref().map(|tz| tz.clone().into()),
                    ),
                    (
                        "wheelchair_boarding",
                        Some(match &stop.wheelchair_boarding {
                            gtfs_structures::Availability::InformationNotAvailable => {
                                "unknown".into()
                            }
                            gtfs_structures::Availability::Available => "available".into(),
                            gtfs_structures::Availability::NotAvailable => "not available".into(),
                        }),
                    ),
                ]
                .into_iter()
                .filter_map(|(key, value)| match value {
                    None => None,
                    Some(v) => Some((key.to_string(), v)),
                })
                .collect::<Map<String, serde_json::Value>>();
                // Add the geometry values
                Feature {
                    geometry: match (&stop.longitude, &stop.latitude) {
                        (Some(lon), Some(lat)) => {
                            Some(geojson::Geometry::new(Point(vec![*lon, *lat])))
                        }
                        _ => None,
                    },
                    id: None,
                    bbox: None,
                    properties: Some(info),
                    foreign_members: None,
                }
            })
            .collect();

        FeatureCollection {
            bbox: None,
            features,
            foreign_members: None,
        }
    }
}

pub mod utility {
    use geojson::FeatureCollection;
    use gtfs_structures::Gtfs;
    use std::fs;
    use std::path::PathBuf;

    /// This function will print all of the stops contained in the GTFS file
    /// # Examples
    /// ```
    /// let gtfs_data = Gtfs::new("tests/gtfs/gtfs_46.zip");
    /// print_stops(gtfs_data);
    /// ```
    ///
    pub fn print_stops(gtfs_data: &Gtfs) {
        println!("They are {} stops in the gtfs", gtfs_data.stops.len());

        for stop in gtfs_data.stops.values() {
            println!("Stop {:?} - {:?} - {:?}", stop.name, stop.id, stop.code);
            println!("Description {:?}", stop.description);

            match &stop.parent_station {
                Option::Some(parent) => println!("Parent station {:?}", parent),
                Option::None => println!("No parent station"),
            }

            match (&stop.longitude, &stop.latitude) {
                (Some(lon), Some(lat)) => println!("Coordinates: {};{}", lon, lat),
                _ => println!("Coordinates not set"),
            }

            match &stop.timezone {
                Option::Some(tmz) => println!("Timezone : {}", tmz),
                _ => println!("No timezone set"),
            }

            match &stop.wheelchair_boarding {
                gtfs_structures::Availability::InformationNotAvailable => {
                    println!("Handicaped access unknown.")
                }
                gtfs_structures::Availability::Available => println!("Handicaped access available"),
                gtfs_structures::Availability::NotAvailable => {
                    println!("Handicaped access unavailable")
                }
            }
            println!("------------------------------");
        }
    }

    /// This function will save the FeatureCollection as a JSON output in the file given to it.
    /// # Examples
    /// ```
    /// let geotype_collection = FeatureCollection::new();
    /// let path = PathBuf::new();
    /// save_to_file(geotype_collection , path);
    /// ```
    pub fn save_to_file(geotype_collection: &FeatureCollection, filename_geo: &PathBuf) {
        println!("{}", geotype_collection);
        fs::write(filename_geo, geotype_collection.to_string()).expect("Unable to write file");
    }
}

fn main() {
    use crate::converter::convert_to_geojson;
    use crate::utility::{print_stops, save_to_file};

    let opt = Opt::from_args();

    if opt.verbose {
        println!("GTFS input file: {:#?}", opt.file);
        println!("GeoJSON output filename: {:#?}", opt.output);
    }

    let gtfs = Gtfs::new(
        opt.file
            .to_str()
            .expect("Invalid file path. Could not convert to string."),
    )
    .expect("The GTFS file is not well formated.");

    if opt.verbose {
        print_stops(&gtfs);
    }

    if opt.verbose {
        println!("Converting the stops to Geotype structures...");
    }

    let stops_as_features = convert_to_geojson(&gtfs, opt.verbose);

    save_to_file(&stops_as_features, &opt.output);
}

#[test]
fn with_code_test() {
    use crate::converter::convert_to_geojson;
    let gtfs = Gtfs::new("test/basic/gtfs/").unwrap();
    let geojson = convert_to_geojson(&gtfs, false);

    let given_feature = &geojson
        .features
        .into_iter()
        .find(|f| f.properties.as_ref().unwrap()["id"].as_str() == Some("stop2"));

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
                "coordinates":[2.449386,48.796058],
                "type":"Point"
                }
        )
    );
}

#[test]
fn no_code_test() {
    use crate::converter::convert_to_geojson;
    let gtfs = Gtfs::new("test/basic/gtfs/").unwrap();
    let geojson = convert_to_geojson(&gtfs, false);

    let given_feature = &geojson
        .features
        .into_iter()
        .find(|f| f.properties.as_ref().unwrap()["id"].as_str() == Some("stop1"));

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
                "coordinates":[2.449386,48.796058],
                "type":"Point"
                }
        )
    );
}
