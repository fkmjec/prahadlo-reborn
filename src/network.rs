use crate::gtfs::*;
use crate::geo_utils::*;

use core::cmp::Ordering;
use strsim::levenshtein;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::Path;

const MAX_PEDESTRIAN_DIST: f32 = 500.0;
const PEDESTRIAN_SPEED: f32 = 1.0;

#[derive(Debug, Clone)]
pub enum Location {
    Stop(String), // Tohle je blbě, je to kvůli tomu zbytečně veliké
    Trip(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub location: Location,
    pub node_id: usize,
    time: u32,
    edges: Vec<usize>,
}

impl Node {
    pub fn new(location: Location, node_id: usize, time: u32) -> Node {
        Node {
            location: location,
            node_id: node_id,
            time: time,
            edges: Vec::new(),
        }
    }

    pub fn get_time(&self) -> u32 {
        self.time
    }

    pub fn get_edges(&self) -> &Vec<usize> {
        &self.edges
    }

    pub fn add_edge(&mut self, node: usize) {
        &self.edges.push(node);
    }

    pub fn get_location(&self) -> &Location {
        &self.location
    }
}

pub struct Connection {
    pub nodes: Vec<Node>,
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.node_id == other.node_id
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        other.get_time().cmp(&self.get_time())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct StopGroup {
    pub name: String,
    pub stops: HashSet<String>,
}

#[derive(Debug)]
pub struct Network {
    stops: HashMap<String, Stop>,
    routes: HashMap<String, Route>,
    trips: HashMap<String, Trip>,
    services: HashMap<String, Service>,
    stop_node_chains: HashMap<String, Vec<usize>>,
    stop_groups: HashMap<String, StopGroup>,
    nodes: Vec<Node>,
}

impl Network {
    pub fn print_debug_info(&self) {
        println!("Number of stops: {}", self.stops.len());
        println!("Number of routes: {}", self.routes.len());
        println!("Number of trips: {}", self.trips.len());
        println!("Number of services: {}", self.services.len());
        println!("Number of nodes: {}", self.nodes.len());
    }

    pub fn get_node(&self, id: usize) -> &Node {
        &self.nodes[id]
    }

    pub fn get_stop(&self, id: &String) -> Option<&Stop> {
        self.stops.get(id)
    }

    pub fn get_trip(&self, id: &String) -> Option<&Trip> {
        self.trips.get(id)
    }

    /// Creates a node, adds it to the node vector, returns the id
    fn create_node(nodes: &mut Vec<Node>, location: Location, time: u32) -> usize {
        let node = Node::new(location, nodes.len(), time);
        let node_id = node.node_id;
        nodes.push(node);
        return node_id;
    }

    fn create_transport_nodes(nodes: &mut Vec<Node>, trips: &HashMap<String, Trip>) {
        // creates transport nodes and the corresponding arrival and departure ones.
        // FIXME extract to a function outside.

        for trip in trips.values() {
            let mut prev_transport: Option<usize> = None;
            let trip_id = trip.trip_id.clone();
            for j in 0..trip.stop_times.len() {
                let stop_time = &trip.stop_times[j];
                let stop_id = stop_time.stop_id.clone();

                let transport: usize = Network::create_node(nodes, Location::Trip(trip_id.clone()), stop_time.departure_time);
                // add edge from previous transport node
                match prev_transport {
                    Some(id) => nodes[id].add_edge(transport),
                    None => (),
                }
                let dep = Network::create_node(nodes, Location::Stop(stop_id.clone()), stop_time.departure_time);
                let arr = Network::create_node(nodes, Location::Stop(stop_id.clone()), stop_time.arrival_time + MINIMAL_TRANSFER_TIME);
                nodes[transport].add_edge(arr);
                nodes[dep].add_edge(transport);
                prev_transport = Some(transport);
            }
        }
    }

    fn sort_node_ids_by_time(nodes: &Vec<Node>, ids: &mut Vec<usize>) -> Vec<usize> {
        ids.sort_by(|a, b| nodes[*a].get_time().cmp(&nodes[*b].get_time()));
        ids.clone()
    }

    /// Adds the departure transfer chain, locks the departure nodes
    fn add_node_chaining(nodes: &mut Vec<Node>, node_ids_by_stop: &mut HashMap<String, Vec<usize>>) {
        let sorted: Vec<Vec<usize>> = node_ids_by_stop.values_mut()
            .map(|ids_by_stop| Network::sort_node_ids_by_time(nodes, ids_by_stop))
            .collect();
        for sorted_ids_at_stop in sorted {
            for index in 0..(sorted_ids_at_stop.len() - 1) {
                let departure = sorted_ids_at_stop[index];
                let arrival = sorted_ids_at_stop[index + 1];
                nodes[departure].add_edge(arrival);
            }
        }
    }

    fn create_node_chains(nodes: &mut Vec<Node>) -> HashMap<String, Vec<usize>> {
        let mut nodes_by_stops: HashMap<String, Vec<usize>> = HashMap::new();
        
        for (index, node) in nodes.iter().enumerate() {
            match &node.location {
                Location::Stop(id) => {
                    match nodes_by_stops.get_mut(id) {
                        Some(vector) => vector.push(index),
                        None => {
                            nodes_by_stops.insert(id.clone(), vec![index]);
                        },
                    };  
                },
                Location::Trip(_) => (),
            }
        }
        Network::add_node_chaining(nodes, &mut nodes_by_stops);
        nodes_by_stops
    }

    /// returns the first departure from stop with id @stop_id after time @time
    fn get_first_departure(&self, stop_id: &String, time: u32) -> Option<usize> {
        let stop_node_chain_opt = self.stop_node_chains.get(stop_id);
        match stop_node_chain_opt {
            Some(stop_node_chain) => Network::bin_search(&self.nodes, time, stop_node_chain),
            None => None,
        }
    }

    fn bin_search(nodes: &Vec<Node>, time: u32, vector: &Vec<usize>) -> Option<usize> {
        let mut l: i32 = 0;
        let mut r = vector.len() as i32 - 1;
        let mut best = None;
        while l <= r {
            let middle = (l + r) / 2;
            let addr = vector[middle as usize];
            if nodes[addr].time >= time {
                best = Some(addr);
                r = middle - 1;
            } else {
                l = middle + 1;
            }
        }
        best
    }

    fn add_pedestrian_connections(nodes: &mut Vec<Node>, stops: &HashMap<String, Stop>, stop_node_chains: &HashMap<String, Vec<usize>>) {
        let coords = get_stop_coords_in_utm(stops);
        let squares = calculate_proximity_squares(&coords, MAX_PEDESTRIAN_DIST);
        let connections = get_pedestrian_connections(stops, &coords, &squares, MAX_PEDESTRIAN_DIST);
        for (stop_id, connection_vector) in connections {
            let empty_ary = vec![];
            let stop_node_ids = stop_node_chains.get(&stop_id).unwrap_or(&empty_ary);
            for (neighbouring_stop_id, dist) in connection_vector {
                let neighbouring_nodes = stop_node_chains.get(&neighbouring_stop_id).unwrap_or(&empty_ary); 
                let travel_time = (dist / PEDESTRIAN_SPEED) as u32;
                for node_id in stop_node_ids {
                    let node_time = nodes[*node_id].time;
                    let dest_node = Network::bin_search(nodes, node_time + travel_time, neighbouring_nodes);
                    match dest_node {
                        Some(id) => {
                            nodes[*node_id].add_edge(id);
                        },
                        None => (),
                    };
                }
            }
        }
    }

    fn get_root_stop_id(stop_id: &String) -> String {
        let mut result = String::from(stop_id);
        for i in 1..stop_id.len() {
            if stop_id.chars().nth(i).unwrap().is_alphabetic() {
                result = String::from(&stop_id[0..i]);
            }
        }
        return result;
    }

    fn create_stop_groups(stops: &HashMap<String, Stop>) -> HashMap<String, StopGroup> {
        let mut result: HashMap<String, StopGroup> = HashMap::new();
        for (stop_id, stop) in stops {
            let root_id = Network::get_root_stop_id(stop_id);
            if let Some(stop_group) = result.get_mut(&root_id) {
                stop_group.stops.insert(stop_id.clone());
            } else {
                let mut stops_in_group = HashSet::new();
                stops_in_group.insert(stop_id.clone());
                result.insert(root_id, StopGroup {name: stop.stop_name.clone(), stops: stops_in_group });
            }
        }
        result
    }
    
    pub fn new(
        path: &Path
    ) -> Network {
        let stops = load_stops(path);
        let routes = load_routes(path);
        let mut trips = load_trips(path);
        let mut services = load_services(path);
        load_service_exceptions(path, &mut services);
        load_stop_times(path, &mut trips);
        let mut nodes = Vec::new();
        let stop_groups = Network::create_stop_groups(&stops);
        Network::create_transport_nodes(&mut nodes, &trips);
        let stop_node_chains = Network::create_node_chains(&mut nodes);
        Network::add_pedestrian_connections(&mut nodes, &stops, &stop_node_chains);

        let nw = Network {
            stops: stops,
            routes: routes,
            trips: trips,
            services: services,
            stop_node_chains: stop_node_chains,
            stop_groups: stop_groups,
            nodes: nodes,
        };

        nw
    }

    /// Compares the names with the supplied name and returns the most similar one (by Levehnstein)
    fn get_stop_group_by_name(&self, name: &String) -> Option<&StopGroup> {
        let mut closest = None;
        let mut best_score = None;
        for (_, g) in &self.stop_groups {
            let score = levenshtein(name.as_str(), g.name.as_str());
            best_score = match best_score {
                Some(past_score) => {
                    if score > past_score {
                        closest = Some(g);
                        Some(score)
                    } else {
                        Some(past_score)
                    }
                },
                None => {
                    closest = Some(g);
                    Some(score)
                }
            }
        }
        closest
    }

    fn get_departures_from_stop_group_after_time(&self, group: &StopGroup, time: u32) -> Vec<&Node> {
        let mut result = Vec::new();
        for stop_id in &group.stops {
            if let Some(dep) = self.get_first_departure(&stop_id, time) {
                result.push(&self.nodes[dep]);
            }
        }
        result
    }

    fn is_destination(&self, node: &Node, dest_stop_group: &StopGroup) -> bool {
        match node.get_location() {
            Location::Stop(stop_id) => {
                dest_stop_group.stops.contains(stop_id)
            },
            Location::Trip(_) => false,
        }
    }
    
    pub fn find_connection(
        &self,
        dep_stop_name: &String,
        dest_stop_name: &String,
        time: u32,
    ) -> Result<Option<Connection>, &str> {
        let mut dists = vec![-1; self.nodes.len()];
        let mut came_from: Vec<i32> = vec![-1; self.nodes.len()];

        // this potentially belongs higher-up in the hierarchy and not in the model
        let start_stop_group = self.get_stop_group_by_name(dep_stop_name).ok_or("Departure stop not found")?;
        let dest_stop_group = self.get_stop_group_by_name(dest_stop_name).ok_or("Destination stop not found")?;
        
        let starts = self.get_departures_from_stop_group_after_time(&start_stop_group, time);
        for s in &starts {
            dists[s.node_id] = time as i32;
        }

        let mut heap = BinaryHeap::from(starts);

        while let Some(node) = heap.pop() {
            if self.is_destination(node, dest_stop_group) {
                let mut index = node.node_id;
                let mut path = Vec::new();
                while came_from[index] != -1 {
                    path.push(self.nodes[index].clone());
                    index = came_from[index] as usize;
                }
                path.reverse();
                return Ok(Some(Connection {nodes: path}));
            }

            for target_node in node.get_edges() {
                let target_node_time = self.nodes[*target_node].get_time() as i32;
                if dists[*target_node] == -1 || (target_node_time < dists[*target_node]) {
                    heap.push(&self.nodes[*target_node]);
                    dists[*target_node] = target_node_time;
                    came_from[*target_node] = node.node_id as i32;
                }
            }
            dists[node.node_id] = node.get_time() as i32;
        }
        return Ok(None);
    }
}
