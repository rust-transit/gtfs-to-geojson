use std::path::PathBuf;

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
