use std::collections::HashMap;

use crate::gtfs::*;

use geo_types::Point;
use proj::Proj;

    
/// Converts stop coordinates in WGS84 to UTM coordinates in zone 33U
pub fn get_stop_coords_in_utm(stops: &HashMap<String, Stop>) -> HashMap<String, Point<f32>> {
    let mut stop_coords: HashMap<String, Point<f32>> = HashMap::new();
    for (stop_id, stop) in stops {
        let from = "EPSG:4326";
        let to = "EPSG:32633";
        let wsg_to_utm = Proj::new_known_crs(&from, &to, None).unwrap();
        let wsg_coords = Point::new(stop.stop_lon, stop.stop_lat);
        let coords = wsg_to_utm.convert(wsg_coords).unwrap();
        stop_coords.insert(stop_id.clone(), coords);
    }
    return stop_coords;
}

/// Takes stop coords in utm and a maximum connections distance. Divides the stops into squares of
/// size max_connection_dist * max_connection_dist.
pub fn calculate_proximity_squares(
    utm_coords: &HashMap<String, Point<f32>>,
    max_connection_dist: f32,
) -> HashMap<(i32, i32), Vec<String>> {
    let mut squares: HashMap<(i32, i32), Vec<String>> = HashMap::new();
    for (stop_id, utm) in utm_coords {
        let square_coords = (
            (utm.x() / max_connection_dist) as i32,
            (utm.y() / max_connection_dist) as i32,
        );
        if squares.contains_key(&square_coords) {
            squares
                .get_mut(&square_coords)
                .unwrap()
                .push(String::from(stop_id));
        } else {
            squares.insert(square_coords, vec![String::from(stop_id)]);
        }
    }
    return squares;
}

/// Takes squares of sizes max_conn_dist times max_conn_dist that contain stops in utm coordinates,
/// and it efficiently computes connections between stops closer than max_conn_dist. (efficiently means faster than
/// O(N^2) N being the number of all stops.
pub fn get_pedestrian_connections(
    stops: &HashMap<String, Stop>,
    utm_coords: &HashMap<String, Point<f32>>,
    squares: &HashMap<(i32, i32), Vec<String>>,
    max_conn_dist: f32,
) -> HashMap<String, Vec<(String, f32)>> {
    let mut connections: HashMap<String, Vec<(String, f32)>> = HashMap::new();
    for ((x, y), stop_ids) in squares {
        for stop_id in stop_ids {
            let coord = utm_coords.get(stop_id).unwrap();
            for dx in -1..2 {
                for dy in -1..2 {
                    if let Some(near_stop_ids) = squares.get(&(x + dx, y + dy)) {
                        for near_id in near_stop_ids {
                            let near_coord = utm_coords.get(near_id).unwrap();
                            let distance = (coord.x() - near_coord.x()).abs()
                                + (coord.y() - near_coord.y()).abs();
                            if distance <= max_conn_dist {
                                if let Some(connection) = connections.get_mut(stop_id) {
                                    connection.push((near_id.clone(), distance));
                                } else {
                                    connections.insert(
                                        String::from(stop_id),
                                        vec![(near_id.clone(), distance)],
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return connections;
}

