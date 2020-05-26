//! This crates aims to be a simple converter for GTFS to GeoJSON formats.

use gtfs_structures::Gtfs;
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

    // To be verbose about what's going on.
    #[structopt(name = "verbose", short = "v", long = "verbose")]
    verbose: bool,
}

pub mod converter {
    use geojson::Value::Point;
    use geojson::{Feature, FeatureCollection};
    use gtfs_structures::Gtfs;
    use serde_json::Map;
    use std::collections::HashSet;

    fn extract_stops(gtfs: &Gtfs, verbose: bool) -> Vec<Feature> {
        // Convert the stops of the GTFS by mapping each field
        gtfs.stops
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
            .collect()
    }

    fn extract_trips_shapes(gtfs: &Gtfs) -> Vec<Feature> {
        let mut shapes_id = HashSet::new();
        gtfs.trips
            .values()
            .filter_map(|trip| {
                trip.shape_id.as_ref().and_then(|shape_id| {
                    if shapes_id.contains(shape_id) {
                        None
                    } else {
                        shapes_id.insert(shape_id);
                        let shape = gtfs.shapes.get(shape_id).map(|shapes| {
                            shapes
                                .iter()
                                .map(|shape| vec![shape.longitude, shape.latitude])
                                .collect::<geojson::LineStringType>()
                        });

                        let geom = shape
                            .map(|geom| geojson::Geometry::new(geojson::Value::LineString(geom)));

                        let properties = gtfs.routes.get(&trip.route_id).map(|route| {
                            let mut properties = Map::new();
                            properties.insert("route_id".to_string(), route.id.clone().into());
                            properties.insert(
                                "route_short_name".to_string(),
                                route.short_name.clone().into(),
                            );
                            properties.insert(
                                "route_long_name".to_string(),
                                route.long_name.clone().into(),
                            );
                            if let Some(color) = route.route_color {
                                properties.insert(
                                    "route_color".to_string(),
                                    format!("{:X}", color).into(),
                                );
                            }
                            if let Some(color) = route.route_text_color {
                                properties.insert(
                                    "route_text_color".to_string(),
                                    format!("{:X}", color).into(),
                                );
                            }
                            properties
                        });
                        Some(Feature {
                            bbox: None,
                            geometry: geom,
                            id: None,
                            properties,
                            foreign_members: None,
                        })
                    }
                })
            })
            .collect()
    }

    /// This function will take a GTFS data format and ouput a FeatureCollection, which can in turn, be printed by the utility module.
    /// If the verbose argument if True, then it will also print each step of conversion.
    /// # Examples
    /// ```
    /// let gtfs_data = Gtfs::new("tests/gtfs/gtfs_46.zip");
    /// convert_to_geojson(gtfs_data, true);
    /// ```
    pub fn convert_to_geojson(gtfs_data: &Gtfs, verbose: bool) -> FeatureCollection {
        let mut features = extract_stops(gtfs_data, verbose);
        let shape_features = extract_trips_shapes(gtfs_data);
        features.extend(shape_features);
        FeatureCollection {
            bbox: None,
            features,
            foreign_members: None,
        }
    }
}

pub mod utility {
    use gtfs_structures::Gtfs;

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
}

fn main() {
    use crate::converter::convert_to_geojson;

    let opt = Opt::from_args();

    if opt.verbose {
        println!("GTFS input file: {:#?}", opt.file);
    }

    let gtfs = Gtfs::new(
        opt.file
            .to_str()
            .expect("Invalid file path. Could not convert to string."),
    )
    .expect("The GTFS file is not well formated.");

    if opt.verbose {
        utility::print_stops(&gtfs);
    }

    if opt.verbose {
        println!("Converting the stops to Geotype structures...");
    }

    let stops_as_features = convert_to_geojson(&gtfs, opt.verbose);

    println!("{}", stops_as_features);
}

#[cfg(test)]
mod test {
    use serde_json::json;

    #[test]
    fn with_code_test() {
        use crate::converter::convert_to_geojson;
        let gtfs = gtfs_structures::Gtfs::new("test/basic/gtfs/").unwrap();
        let geojson = convert_to_geojson(&gtfs, false);

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
        let geojson = convert_to_geojson(&gtfs, false);

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
        let geojson = convert_to_geojson(&gtfs, false);

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
                "route_color": "RGB { #000000 }",
                "route_text_color": "RGB { #FFFFFF }",
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
}
