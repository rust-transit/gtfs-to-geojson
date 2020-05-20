use std::fs;
use geojson::{Feature, FeatureCollection, Geometry, Value};
use gtfs_structures::Gtfs;
use serde_json::Map;
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
// This test aims to make sure that the GTFS and the GeoJson files are equivalent.
fn simple_conversion() {
    // read a gtfs
    let gtfs = Gtfs::new("test/gtfs/gtfs_46.zip").unwrap();
    let geojson = convert_to_geojson(&gtfs, false);

    let default_err = String::from("Errrrrrr");
    let default_err_value = serde_json::to_value(&default_err).unwrap();

    // Make sure that we have as many fields as in the GTFS
    assert_eq!(geojson.features.len(), gtfs.stops.len());

    // Make sure that for every element of the geojson,
    // we have the same..

    // name
    let gtfs_name = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .name;


    let geojson_name = geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("name")
                        .expect("Name has no value")
                        .as_str()
                        .expect("Name is not a string");

    assert_eq!( gtfs_name, geojson_name);

    // id

    let gtfs_id = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .id;
    let geojson_id = geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("id")
                        .expect("Id has no value")
                        .as_str()
                        .expect("Name is not a string");

    assert_eq!(gtfs_id, geojson_id );

    // code
    let gtfs_code = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .code
                        .as_ref()
                        .unwrap_or(&default_err)
                        .as_str();
    let geojson_code = &geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("code")
                        .unwrap_or(&default_err_value)
                        .as_str()
                        .expect("code is not a string");

    assert_eq!(gtfs_code, geojson_code );

    // description
    let gtfs_descrip = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .description;
    let geojson_descrip = geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("description")
                        .expect("description has no value")
                        .as_str()
                        .expect("description is not a string");

    assert_eq!(gtfs_descrip, geojson_descrip );

    // parent station
    let gtfs_parent_station = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .parent_station
                        .as_ref()
                        .unwrap_or(&default_err)
                        .as_str();

    let geojson_parent_station = geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("parent_station")
                        .unwrap_or(&default_err_value)
                        .as_str()
                        .expect("description is not a string");

    assert_eq!(gtfs_parent_station, &geojson_parent_station );

    //longitude and latitude
    // geometry: match (&stop.longitude, &stop.latitude) {
    //     (Some(lon), Some(lat)) => Some(Geometry::new(Value::Point(vec![*lon, *lat]))),
    //     _ => None,
    // },
    let gtfs_lat = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .latitude
                        .as_ref()
                        .unwrap();
    let geojson_lat = &geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .geometry
                        .as_ref()
                        .unwrap()
                        .value;

    let geojson_lat_val = match geojson_lat {
        Value::Point(v) => v,
        _ => panic!("No value for latitude")
    };

    assert_eq!(gtfs_lat, &&geojson_lat_val[1]);

    let gtfs_long = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .longitude
                        .as_ref()
                        .unwrap();

    let geojson_long = &geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .geometry
                        .as_ref()
                        .unwrap()
                        .value;

    let geojson_long_val = match geojson_long {
        Value::Point(v) => v,
        _ => panic!("No value for latitude")
    };

    assert_eq!(gtfs_long, &&geojson_long_val[0]);

    // timezone
    let gtfs_tz = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .timezone
                        .as_ref()
                        .unwrap_or(&default_err)
                        .as_str();

    let geojson_tz = geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("timezone")
                        .unwrap_or(&default_err_value)
                        .as_str()
                        .expect("description is not a string");

    assert_eq!(gtfs_tz, &geojson_tz );

    // wheelchair boarding
    let gtfs_wheelchair = &gtfs.stops.values().nth(0)
                        .expect("The GTFS does not have a name")
                        .wheelchair_boarding;
    let gtfs_wheelchair_val = match gtfs_wheelchair {
        gtfs_structures::Availability::InformationNotAvailable => "unknown",
        gtfs_structures::Availability::Available => "available",
        gtfs_structures::Availability::NotAvailable => "not available",
    };

    let geojson_wheelchair = geojson.features
                        .first()
                        .expect("The GeoJson feature does not exist")
                        .properties
                        .as_ref()
                        .expect("The property has no information")
                        .get("wheelchair_boarding")
                        .unwrap_or(&default_err_value)
                        .as_str()
                        .expect("description is not a string");

    assert_eq!(gtfs_wheelchair_val, geojson_wheelchair );

}

#[test]
#[should_panic]
fn simple_conversion_panic(){
    panic!("Whoops");
}
