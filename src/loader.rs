use crate::model::gtfs::*;
use crate::model::network::*;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use geo_types::Point;
use proj::Proj;

const MAX_PEDESTRIAN_DIST: f32 = 500.0;
const PEDESTRIAN_SPEED: f32 = 3.6;
const BASE_PEDESTRIAN_TRANSFER_TIME: f32 = 60.0;

/// Converts stop coordinates in WGS84 to UTM coordinates in zone 33U
fn get_stop_coords_in_utm(stops: &HashMap<String, Rc<Stop>>) -> HashMap<String, Point<f32>> {
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
fn calculate_proximity_squares(
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
fn get_pedestrian_connections(
    stops: &HashMap<String, Rc<Stop>>,
    utm_coords: &HashMap<String, Point<f32>>,
    squares: &HashMap<(i32, i32), Vec<String>>,
    max_conn_dist: f32,
) -> HashMap<String, Vec<(Rc<Stop>, f32)>> {
    let mut connections: HashMap<String, Vec<(Rc<Stop>, f32)>> = HashMap::new();
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
                            if (distance <= max_conn_dist) {
                                if let Some(connection) = connections.get_mut(stop_id) {
                                    connection.push((stops.get(near_id).unwrap().clone(), distance));
                                } else {
                                    connections.insert(
                                        String::from(stop_id),
                                        vec![(stops.get(near_id).unwrap().clone(), distance)],
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

/// Creates a node, adds it to the node vector, returns the id
fn create_node(nodes: &mut Vec<Node>, location: Location, time: u32) -> usize {
    let node = Node::new(location, nodes.len(), time);
    let node_id = node.node_id;
    nodes.push(node);
    return node_id;
}

/// Loads the entire transport network from a GTFS directory. Optimized for the Prague usecase
pub fn load_transport_network(path: &Path) -> Network {
    let loader_start = SystemTime::now();
    println!("Started loader!");
    // Huge init with all the needed data structures
    // Stops saved here to be used for processing. They will be thrown away after loading is complete.
    let mut stops_raw = load_stops(path);
    println!("Stop loading is done!, {}", loader_start.elapsed().unwrap().as_secs());
    let routes = load_routes(path);
    println!("Route loading is done!, {}", loader_start.elapsed().unwrap().as_secs());
    let mut services = load_services(path);
    println!("Service loading is done!, {}", loader_start.elapsed().unwrap().as_secs());
    load_service_exceptions(path, &mut services);
    println!("Exception loading is done!, {}", loader_start.elapsed().unwrap().as_secs());
    // Same as with stops, these trips shall be thrown away after processing.
    // FIXME possible optimization here with Rc<RefCell<>>.
    let mut trips = load_trips(path);
    println!("Trip loading is done!, {}", loader_start.elapsed().unwrap().as_secs());
    load_stop_times(path, &mut trips);
    println!("StopTime loading is done!, {}", loader_start.elapsed().unwrap().as_secs());

    println!("Raw loading is done!, {}", loader_start.elapsed().unwrap().as_secs());

    let mut nodes: Vec<Node> = Vec::new();
    let mut arrival_nodes: HashMap<String, usize> = HashMap::new();

    // creates transport nodes and the corresponding arrival and departure ones.
    // FIXME extract to a function outside.
    for trip in trips.values() {
        let mut prev_transport: Option<usize> = None;
        let trip_id = trip.trip_id;
        for i in 0..trip.stop_times.len() {
            let stop_time = &trip.stop_times[i];
            let stop_id = stop_time.stop_id;

            let mut transport: usize = create_node(&mut nodes, Location::Trip(trip_id.clone()), stop_time.departure_time);
            // add edge from previous transport node
            match prev_transport {
                Some(id) => nodes[id].add_edge(&transport),
                None => (),
            }
            let mut dep = create_node(&mut nodes, Location::Stop(stop_id.clone()), stop_time.departure_time);
            //stops_raw.get_mut(&trip.stop_times[i].stop_id).unwrap().add_dep_node(&dep);
            let mut arr = create_node(&mut nodes, Location::Stop(stop_id.clone()), stop_time.arrival_time + MINIMAL_TRANSFER_TIME);
            arrival_nodes.insert(stop_id.clone(), arr);
            nodes[transport].add_edge(&arr);
            nodes[dep].add_edge(&transport);

            prev_transport = Some(transport);
        }
    }

    println!("Node creation is done!, {}", loader_start.elapsed().unwrap().as_secs());
    println!("Connecting arrival nodes to departure node chains");
    let mut stops: HashMap<String, Rc<Stop>> = HashMap::new();
    // Fix nodes as immutable and create a hashmap of pointers to them
    for stop in stops_raw.values_mut() {
        stop.add_dep_node_chaining(&mut nodes);
        let stop_rc = Rc::new(stop.clone());
        for dep_node_id in stop_rc.get_dep_nodes() {
            nodes[*dep_node_id].stop = Some(stop_rc.clone());
        }
        stops.insert(stop.stop_id.clone(), stop_rc);
    }

    for (stop_id, arr_node_id) in arrival_nodes {
        // FIXME error handling
        let stop = stops.get(&stop_id).unwrap();
        let first_dep = stop.get_earliest_dep(nodes[arr_node_id].get_time(), &nodes).unwrap();
        match first_dep {
            Some(dep_node_id) => nodes[arr_node_id].add_edge(&dep_node_id),
            None => (),
        }
        nodes[arr_node_id].stop = Some(stop.clone());
        // TODO add pedestrian transfers
    }
    println!("Arrival node chains are complete!, {}", loader_start.elapsed().unwrap().as_secs());

    println!("Calculating pedestrian connections...");
    let utm_coords = get_stop_coords_in_utm(&stops_raw);
    let squares = calculate_proximity_squares(&utm_coords, MAX_PEDESTRIAN_DIST);
    let connections = get_pedestrian_connections(&stops, &utm_coords, &squares, MAX_PEDESTRIAN_DIST);
    
    let mut new_arr = Vec::new();
    for arr_id in &arrival_nodes {
        new_arr.append(&mut node_add_pedestrian_connections(&mut nodes, &connections, *arr_id));
    }
    arrival_nodes.append(&mut new_arr);
    
    println!("Adding edges between arrival and departure nodes...");
    for arr_id in &arrival_nodes {
        let stop = nodes[*arr_id].stop.as_ref().unwrap(); // arr node must have a stop
        match stop.get_earliest_dep(nodes[*arr_id].get_time(), &nodes) {
            Some(dep_id) => nodes[*arr_id].add_edge(&dep_id),
            None => (),
        }
    }

    return Network::new(stops, routes, trips, services, nodes);
}
