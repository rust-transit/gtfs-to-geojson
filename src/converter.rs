use geojson::Value::Point;
use geojson::{Feature, FeatureCollection};
use gtfs_structures::Gtfs;
use serde_json::Map;
use std::collections::HashSet;

fn extract_stops(gtfs: &Gtfs) -> Vec<Feature> {
    // Convert the stops of the GTFS by mapping each field
    gtfs.stops
        .values()
        .map(|stop| {
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
                        gtfs_structures::Availability::Unknown(u) => u.to_string().into(),
                    }),
                ),
            ]
            .into_iter()
            .filter_map(|(key, value)| value.map(|v| (key.to_string(), v)))
            .collect::<Map<String, serde_json::Value>>();
            // Add the geometry values
            Feature {
                geometry: match (&stop.longitude, &stop.latitude) {
                    (Some(lon), Some(lat)) => Some(geojson::Geometry::new(Point(vec![*lon, *lat]))),
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
    // The HashSet will contain shape_id already treated, to avoid duplicated features
    let mut shapes_id = HashSet::new();
    gtfs.trips
        .values()
        .filter_map(|trip| -> Option<Feature> {
            match trip.shape_id {
                Some(_) => {
                    trip.shape_id.as_ref().and_then(|shape_id| {
                        if shapes_id.insert(shape_id) {
                            // new shape found
                            Some(get_new_feature_from_shape(gtfs, shape_id, trip))
                        } else {
                            // shape_id was already treated
                            None
                        }
                }
            )},
                None => {
                    if shapes_id.insert(&trip.route_id) {
                        Some(get_straight_line_feature_between_stops(gtfs, trip))
                    } else {
                        None
                    }
                },
            }
        })
        .collect::<Vec<Feature>>()
}

fn get_new_feature_from_shape(
    gtfs: &Gtfs,
    shape_id: &str,
    trip: &gtfs_structures::Trip,
) -> Feature {
    let shape = gtfs.shapes.get(shape_id).map(|shapes| {
        // create a Vec<Position>, aka a LineStringType
        shapes
            .iter()
            .map(|shape| vec![shape.longitude, shape.latitude])
            .collect::<geojson::LineStringType>()
    });

    let geom = shape.map(|geom| geojson::Geometry::new(geojson::Value::LineString(geom)));
    let properties = get_route_properties(gtfs, &trip.route_id);
    Feature {
        bbox: None,
        geometry: geom,
        id: None,
        properties,
        foreign_members: None,
    }
}

fn get_straight_line_feature_between_stops(
    gtfs: &Gtfs,
    trip: &gtfs_structures::Trip
) -> Feature {
    // let geom = trip.stop_times.iter().map(|stop_time| vec![stop_time.stop.longitude, stop_time.stop.latitude]).collect::<geojson::LineStringType>();
    let shape = trip.stop_times
        .iter()
        .map(|stop_time| vec![stop_time.stop.longitude.unwrap(), stop_time.stop.latitude.unwrap()])
        .collect::<geojson::LineStringType>();
    let geom = geojson::Geometry::new(geojson::Value::LineString(shape));
    let properties = get_route_properties(gtfs, &trip.route_id);
    Feature {
        bbox: None, 
        geometry: Some(geom), 
        id: None, 
        properties, 
        foreign_members: None
    }
}


// Given a GTFS reference and a route_id reference, outputs useful properties from the route.
fn get_route_properties(
    gtfs: &Gtfs,
    route_id: &str,
) -> Option<Map<String, serde_json::value::Value>> {
    gtfs.routes.get(route_id).map(|route| {
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
        properties.insert("route_color".to_string(), format!("{}", route.color).into());
        properties.insert(
            "route_text_color".to_string(),
            format!("{}", route.text_color).into(),
        );
        properties
    })
}

/// This function will take a GTFS data format and ouput a FeatureCollection, which can in turn, be printed by the utility module.
/// # Examples
/// ```
/// let gtfs_data = Gtfs::new("tests/gtfs/gtfs_46.zip");
/// convert_to_geojson(gtfs_data, true);
/// ```
pub fn convert_to_geojson(gtfs_data: &Gtfs) -> FeatureCollection {
    let mut features = extract_stops(gtfs_data);
    let shape_features = extract_trips_shapes(gtfs_data);
    features.extend(shape_features);
    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
}
