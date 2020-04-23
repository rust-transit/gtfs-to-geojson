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
                        gtfs_structures::Availability::InformationNotAvailable => "unknown".into(),
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
                    (Some(lon), Some(lat)) => Some(Geometry::new(Value::Point(vec![*lon, *lat]))),
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

fn save_to_file(geotype_collection: &FeatureCollection, filename_geo: &PathBuf) {
    println!("{}", geotype_collection);

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
