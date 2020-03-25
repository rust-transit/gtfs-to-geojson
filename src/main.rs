use gtfs_structures::Gtfs;
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
}

fn main() {
    let opt = Opt::from_args();
    println!("GTFS input file: {:#?}", opt.file);
    println!("GeoJSON output filename: {:#?}", opt.output);
    println!("");

    let gtfs = Gtfs::new(opt.file.to_str().unwrap()).expect("The GTFS file is not well formated.");
    println!("They are {} stops in the gtfs", gtfs.stops.len());
    for stop in gtfs.stops.values() {
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
