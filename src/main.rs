use geojson::{Feature, FeatureCollection, Geometry, Value};
use gtfs_structures::Gtfs;
use serde_json::Map;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]

struct Opt {
    // GTFS files
    #[structopt(name = "gtfs", short, long, parse(from_os_str))]
    file: PathBuf,

    // Output file
    #[structopt(name = "json", short, long, parse(from_os_str))]
    output: PathBuf,

    // To be verbose about what's going on.
    #[structopt(name = "verbose", short, long)]
    verbose: bool,

    #[structopt(name = "print-only", short, long)]
    print_only: bool,
}

fn print_stops(gtfs_data: &Gtfs) {
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

fn convert_to_geojson(gtfs_data: &Gtfs, verbose: bool) -> FeatureCollection {
    // For every stop
    // We create a geo_types::Point
    // We add it to a geojson::Feature as the .geometry field
    // We also create a Map<String, JsonValue>, added to the .properties filed
    // of the geojson::Feature.
    // We .push that geojson::Feature to the FeatureCollection,

    // Return

    let mut stops_features = FeatureCollection {
        bbox: None,
        features: vec![],
        foreign_members: None,
    };

    for stop in gtfs_data.stops.values() {
        let mut stop_info = Map::new();
        if verbose {
            println!("Stop {:?} - {:?} - {:?}", stop.name, stop.id, stop.code);
            println!("Description {:?}", stop.description);
        }

        // Build the info of the stop
        stop_info.insert("name".to_string(), stop.name.to_string().into());
        stop_info.insert("id".to_string(), stop.id.to_string().into());
        stop_info.insert(
            "description".to_string(),
            stop.description.to_string().into(),
        );

        match &stop.code {
            Option::Some(code) => stop_info.insert("code".to_string(), code.to_string().into()),
            Option::None => None,
        };

        match &stop.parent_station {
            Option::Some(parent) => {
                stop_info.insert("parent_station".to_string(), parent.to_string().into())
            }
            Option::None => None,
        };

        match &stop.timezone {
            Option::Some(tmz) => stop_info.insert("timezone".to_string(), tmz.to_string().into()),
            _ => None,
        };

        match &stop.wheelchair_boarding {
            gtfs_structures::Availability::InformationNotAvailable => {
                stop_info.insert("wheelchair_boarding".to_string(), "unknown".into());
            }
            gtfs_structures::Availability::Available => {
                stop_info.insert("wheelchair_boarding".to_string(), "available".into());
            }
            gtfs_structures::Availability::NotAvailable => {
                stop_info.insert("wheelchair_boarding".to_string(), "not available".into());
            }
        };

        // Add the geometry values
        stops_features.features.push(Feature {
            geometry: match (&stop.longitude, &stop.latitude) {
                (Some(lon), Some(lat)) => Some(Geometry::new(Value::Point(vec![*lon, *lat]))),
                _ => None,
            },
            id: None,
            bbox: None,
            properties: Some(stop_info),
            foreign_members: None,
        });
    }

    return stops_features;
}

fn save_to_file(geotype_collection: &FeatureCollection, filename_geo: &PathBuf) {
    fs::write(filename_geo, geotype_collection.to_string()).expect("Unable to write file");
}

fn main() {
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
    print_stops(&gtfs);

    if opt.verbose {
        println!("Converting the stops to Geotype structures...");
    }

    let stops_as_features = convert_to_geojson(&gtfs, opt.verbose);

    save_to_file(&stops_as_features, &opt.output);
}
