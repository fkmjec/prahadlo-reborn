use crate::gtfs::*;
use core::cmp::Ordering;
use serde::Deserialize;
use std::collections::{BinaryHeap, HashMap};
use std::path::Path;

pub static MINIMAL_TRANSFER_TIME: u32 = 0;

/// This is an entry point .
struct TransferPoint {
    departure_nodes: Vec<usize>,
}

impl TransferPoint {
    pub fn add_dep_node(&mut self, dep_node: usize) {
        self.departure_nodes.push(dep_node);
    }

    /// Adds the departure transfer chain, locks the departure nodes
    pub fn add_dep_node_chaining(&mut self, nodes: &mut Vec<Node>) {
        if self.departure_nodes.len() > 1 {
            self.departure_nodes
                .sort_by(|a, b| nodes[*a].get_time().cmp(&nodes[*b].get_time()));
            for index in 0..self.departure_nodes.len() - 2 {
                let dep = self.departure_nodes[index];
                let arr = self.departure_nodes[index + 1];
                nodes[dep].add_edge(arr);
            }
        }
    }

    pub fn get_earliest_dep(
        &self,
        arr_time: u32,
        nodes: &Vec<Node>,
    ) -> Option<usize> {
        let mut l: i32 = 0;
        let mut r = self.departure_nodes.len() as i32 - 1;
        let mut best = None;
        while l <= r {
            let middle = (l + r) / 2;
            let addr = self.departure_nodes[middle as usize];
            if nodes[addr].get_time() >= arr_time {
                best = Some(addr);
                r = middle - 1;
            }
            if nodes[self.departure_nodes[middle as usize]].get_time() < arr_time {
                l = middle + 1;
            }
        }
        best
    }
}

#[derive(Debug, Clone)]
pub enum Location {
    Stop(String),
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
pub struct Network {
    stops: HashMap<String, Stop>,
    routes: HashMap<String, Route>,
    trips: HashMap<String, Trip>,
    services: HashMap<String, Service>,
    hop_on_points: HashMap<String, Vec<usize>>,
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
                let mut dep = Network::create_node(nodes, Location::Stop(stop_id.clone()), stop_time.departure_time);
                let mut arr = Network::create_node(nodes, Location::Stop(stop_id.clone()), stop_time.arrival_time + MINIMAL_TRANSFER_TIME);
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
    fn add_dep_node_chaining(nodes: &mut Vec<Node>, node_ids_by_stop: &mut HashMap<String, Vec<usize>>) {
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

    pub fn new(
        path: &Path
    ) -> Network {
        let stops = load_stops(path);
        let routes = load_routes(path);
        let mut trips = load_trips(path);
        let mut services = load_services(path);
        let service_exceptions = load_service_exceptions(path, &mut services);
        let stop_times = load_stop_times(path, &mut trips);
        let mut nodes = Vec::new();

        Network::create_transport_nodes(&mut nodes, &trips);

        let mut nw = Network {
            stops: stops,
            routes: routes,
            trips: trips,
            services: services,
            hop_on_points: HashMap::new(),
            nodes: nodes,
        };
        

        nw
    }
    /*
    pub fn find_connection(
        &self,
        dep_stop_id: &str,
        target_stop_id: &str,
        time: u32,
    ) -> Result<Option<u32>, &str> {
        let mut dists = vec![-1; self.nodes.len()];
        let mut came_from: Vec<i32> = vec![-1; self.nodes.len()];
        let mut heap = BinaryHeap::new();
        let start = self
            .stops
            .get(dep_stop_id)
            .ok_or("Stop not found.")?
            .get_earliest_dep(time, &self.nodes)?
            .ok_or("There is no departure from the stop after the selected time")?;
        dists[start] = time as i32;
        heap.push(start);

        while let Some(popped) = heap.pop() {
            println!("POPPED! {:?}", self.nodes[popped]);
            let node_struct = &self.nodes[popped];
            if node_struct.stop.borrow().stop_id.as_str() == target_stop_id {
                let mut index = popped;
                while came_from[index] != -1 {
                    println!("{:?}", self.nodes[index]);
                    index = came_from[index] as usize;
                }
                return Ok(Some(node_struct.get_time() - time));
            }
            for edge in node_struct.get_edges() {
                if dists[edge.target_node] == -1 || ((node_struct.get_time() + edge.cost()) as i32) < dists[edge.target_node] {
                    heap.push(edge.target_node); // TODO solve this inefficient bullcrap
                    dists[edge.target_node] = (node_struct.get_time() + edge.cost()) as i32;
                    came_from[edge.target_node] = popped as i32;
                }
            }
            dists[popped] = node_struct.get_time() as i32;
        }
        return Ok(None);
<<<<<<< HEAD
    } */
}
