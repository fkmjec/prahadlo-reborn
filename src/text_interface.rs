use std::process::exit;

use crate::network::*;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use chrono::NaiveDateTime;

const datetime_format: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Invalid,
    GetConnection(NaiveDateTime, String, String),
    PrintNode(usize),
    PrintStop(String),
    PrintTrip(String),
}

pub struct TextInterface {
    rl: Editor<()>,
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
        Location::Trip(trip_id, _) => println!(" - corresponding to trip {}", trip_id),
    };
}

fn print_node(node: &Node) {
    println!("Node with id {}, time {}:", node.node_id, get_time_string(node.get_time()));
    print_location(node.get_location());
    println!(" - Edges to nodes {:?}", node.get_edges());
}

fn print_connection(nw: &Network, conn: &Connection) {
    let mut index = 0;
    loop {
        // Go through all the waiting stops at the beginning of the connection
        match conn.nodes[index].location {
            Location::Trip(_, _) => break,
            _ => index = index + 1,
        }
    }

    let mut past_node = &conn.nodes[index-1];
    for node in &conn.nodes[index..] {
        let hours = node.get_time() / 3600;
        let minutes = (node.get_time() - hours * 3600) / 60;
        match node.get_location() {
            Location::Stop(stop_id1) => {
                match past_node.get_location() {
                    Location::Stop(stop_id2) => {
                        if stop_id1 != stop_id2 {
                            print!("{} -> ", get_time_string(node.get_time()));
                            println!("pedestrian transfer from stop {} to stop {}", stop_id1, stop_id2);
                        }
                    },
                    Location::Trip(trip_id, _) => {
                        print!("{}:{} -> ", hours, minutes);
                        println!("getting off from trip {} at stop {}", trip_id, stop_id1);
                    }
                }
            },
            Location::Trip(trip_id1, _) => {
                match past_node.get_location() {
                    Location::Stop(stop_id) => {
                        print!("{}:{} -> ", hours, minutes);
                        println!("boarding trip {} at stop {}", trip_id1, stop_id);
                    },
                    Location::Trip(trip_id2, _) => {},
                }
            },
        }
        past_node = node;
    }
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

fn parse_connection(conn_details: &String) -> Command {
    let args: Vec<&str> = conn_details.split("|").map(|x| x.trim()).collect();
    if args.len() == 3 {
        let time_res = NaiveDateTime::parse_from_str(args[0], datetime_format);
        let dep_stop_id = String::from(args[1]);
        let dest_stop_id = String::from(args[2]);
        match time_res {
            Ok(time) => Command::GetConnection(time, dep_stop_id, dest_stop_id),
            Err(_) => Command::Invalid,
        }
    } else {
        Command::Invalid
    }
}

fn command_from_line(line: &str) -> Command {
    let complete_input: Vec<&str> = line.trim().split(" ").collect();
    let command_type = complete_input[0];
    let args = &complete_input[1..];

    // TODO: add validation of command arguments
    match command_type {
        "node" => parse_print_node(args),
        "stop" => parse_print_stop(args),
        "trip" => parse_print_trip(args),
        "conn" => parse_connection(&args.join(" ")),
        "help" => Command::Help,
        _ => Command::Invalid,
    }
}

fn print_help() {
    println!("Commands:");
    println!(" - node [node_id] - prints information about a node with the id");
    println!(" - stop [stop_id] - prints information about a stop with the id");
    println!(" - trip [trip_id] - prints information about a trip with the id");
    println!(" - conn [time] | [stop_name_1] | [stop_name_2] - finds a connection between the stops. \n [time] is in the format YYYY-MM-DD HH:MM:SS");
}

fn print_invalid() {
    println!("The command you entered was incorrect!");
}

impl TextInterface {
    pub fn new(history_file: &str) -> TextInterface {
        let mut rl = Editor::<()>::new();
        if rl.load_history(history_file).is_err() {
            println!("No previous history.");
        }
        TextInterface { rl: rl }
    }

    fn get_command(&mut self) -> Command {
        let readline = self.rl.readline(">> ");
        match readline {
            Ok(line) => {
                self.rl.add_history_entry(line.as_str());
                let command = command_from_line(&line);
                command        
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                exit(0);
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                exit(0);
            },
            Err(err) => {
                println!("Error: {:?}", err);
                Command::Invalid
            }
        }
    }

    pub fn process_command(&mut self, nw: &Network) {
        let cmd = self.get_command();
        match cmd {
            Command::PrintNode(id) => {
                let node = nw.get_node(id);
                print_node(node);
            },
            Command::PrintStop(id) => {
                match nw.get_stop(&id) {
                    Some(stop) => println!("{:?}", stop),
                    None => println!("ERROR: no stop with such id")
                }
            },
            Command::PrintTrip(id) => {
                match nw.get_trip(&id) {
                    Some(trip) => println!("{:?}", trip),
                    None => println!("ERROR: no trip with such id")
                }
            },
            Command::GetConnection(time, s1, s2) => {
                let lookup_result = nw.find_connection(&s1, &s2, time);
                match lookup_result {
                    Ok(maybe_connection) => {
                        match maybe_connection {
                            Some(conn) => print_connection(nw, &conn),
                            None => println!("No connection found, sorry!"),
                        }
                    },
                    Err(err_string) => println!("{}", err_string),
                }
            },
            Command::Help => print_help(),
            Command::Invalid => print_invalid(),
        } 
    }
}