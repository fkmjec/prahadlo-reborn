use std::io;
use std::io::Write;

use crate::network::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Invalid,
    PrintNode(usize),
    PrintStop(String),
    PrintTrip(String),
}

pub fn get_time_string(time_in_seconds: u32) -> String {
    let hours = time_in_seconds / 3600;
    let minutes = (time_in_seconds % 3600) / 60;
    let seconds = time_in_seconds % 60;
    format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
}

fn print_location(location: &Location) {
    match location {
        Location::Stop(stop_id) => println!(" - corresponding to stop {}", stop_id),
        Location::Trip(trip_id) => println!(" - corresponding to trip {}", trip_id),
    };
}

fn print_node(node: &Node) {
    println!("Node with id {}, time {}:", node.node_id, get_time_string(node.get_time()));
    print_location(node.get_location());
    println!(" - Edges to nodes {:?}", node.get_edges());
}

fn parse_print_node(args: &[&str]) -> Command {
    if args.len() == 1 {
        let node_id = args[0].parse::<usize>();
        match node_id {
            Ok(id) => Command::PrintNode(id),
            Err(_) => Command::Invalid,
        }
    } else {
        Command::Invalid
    }
}

fn parse_print_stop(args: &[&str]) -> Command {
    if args.len() == 1 {
        let stop_id = args[0];
        Command::PrintStop(String::from(stop_id))
    } else {
        Command::Invalid
    }
}

fn parse_print_trip(args: &[&str]) -> Command {
    if args.len() == 1 {
        let trip_id = args[0];
        Command::PrintTrip(String::from(trip_id))
    } else {
        Command::Invalid
    }
}

fn command_from_line(line: &str) -> Command {
    let args: Vec<&str> = line.trim().split(" ").collect();

    // TODO: add validation of command arguments
    match args[0] {
        "node" => parse_print_node(&args[1..]),
        "stop" => parse_print_stop(&args[1..]),
        "trip" => parse_print_trip(&args[1..]),
        "help" => Command::Help,
        _ => Command::Invalid,
    }
}

fn print_help() {
    println!("Commands:");
    println!(" - node [node_id] - prints information about a node with the id");
    println!(" - stop [stop_id] - prints information about a stop with the id");
    println!(" - trip [trip_id] - prints information about a trip with the id");
}

fn print_invalid() {
    println!("The command you entered was incorrect!");
}

fn get_command() -> Command {
    let mut line = String::new();
    print!("> ");
    io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut line).unwrap();
    let mut command = command_from_line(&line);
    while command == Command::Invalid || command == Command::Help {
        if command == Command::Invalid {
            print_invalid();
        } else {
            print_help();
        }
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();    
        command = command_from_line(&line);
    }
    command
}

pub fn process_command(nw: &Network) {
    let cmd = get_command();
    match cmd {
        Command::PrintNode(id) => {
            let node = nw.get_node(id);
            print_node(node);
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