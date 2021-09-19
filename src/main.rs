use std::path::Path;

mod gtfs;
mod network;
mod text_interface;

use network::*;
use text_interface::*;

fn main() {
    println!("Hello, world! Prahadlo here!");
    let nw = Network::new(Path::new("data/"));
    nw.print_debug_info();
    loop {
        let cmd = get_command();
        match cmd {
            Command::PrintNode(id) => {
                let node = nw.get_node(id);
                node.print_description();
            },
            Command::PrintStop(id) => {
                match nw.get_stop(id) {
                    Some(stop) => println!("{:?}", stop),
                    None => println!("ERROR: no stop with such id")
                }
            },
            Command::PrintTrip(id) => {
                match nw.get_trip(id) {
                    Some(trip) => println!("{:?}", trip),
                    None => println!("ERROR: no trip with such id")
                }
            },
            _ => (), 
        }    
    }
}
