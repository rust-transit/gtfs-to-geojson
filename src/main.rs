use std::path::PathBuf;
use structopt::StructOpt;
use gtfs_structures::Gtfs;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]

struct Opt {

    // GTFS files
    #[structopt(name = "fichier gtfs", parse(from_os_str))]
    file: PathBuf,

    // Output file
    #[structopt(name = "fichier geojson de sortie", short, long, parse(from_os_str))]
    output: PathBuf,

}

fn main() {
    let opt = Opt::from_args();
    println!("Fichier GTFS : {:#?}", opt.file);
    println!("Fichier de sortie : {:#?}", opt.output);

    let gtfs = gtfs_structures::Gtfs::new(opt.file.to_str().unwrap()).expect("Le fichier GTFS n'est pas valable.");
    println!("there are {} stops in the gtfs", gtfs.stops.len());
    for () in gtfs.stops.iter() {
        println!("Arrêt {:?} - {:?} - {:?}", stop.name ,stop.id, stop.code);
        println!("{:?}", stop.description );

        match stop.parent_station {
            Option::Some(String) => println!("{:?}", stop.parent_station),
            Option::None         => println!("Pas de station parent."),
        }

        match (stop.longitude, stop.latitude) {
            Option::(Some(long),Some(lat)) => println!("{}°N, {}°E", long, lat),
            _ => println("Pas de coordonnées pour cette station."),
        }

        match stop.timezone {
            Option::Some(tmz) => println("Fuseau horaire : {}", tmz ),
            _                 => println("Pas de fuseau hoaire."),
        }

        match stop.wheelchair_boarding {
            Option::InformationNotAvailable(s) => println!("Accès PMA inconnu."),
            Option::Available(s)    => println!("Accès PMA."),
            Option::NotAvailable(s) => println!("Pas d'accès PMA."),
        }

        println!("------------------------------");
    }
}

// fn import_gtfs(&str path){
//
// }
//
// fn convert_to_geojson(gtfs_structures gt){
//
// }
