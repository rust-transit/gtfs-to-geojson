
use gtfs_structures::Gtfs;
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
pub fn save_to_file(geotype_collection: &geojson::FeatureCollection, filename_geo: &PathBuf) {
    std::fs::write(filename_geo, geotype_collection.to_string()).expect("Unable to write file");
}
