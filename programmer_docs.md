# Prahadlo - programmer's documentation
Prahadlo is a tool for looking up connections in public transport in Prague. It is written in Rust.

The program is divided into modules:
  * `gtfs`
  This module contains all the GTFS structures needed for deserializing the input [data](http://data.pid.cz/PID_GTFS.zip) from PID.
  * `network`
  This is where the data model needed for the lookups lives and where all the algorithms are located. If this app was a fully-fledged Model-View-Controller, this would be the Model.
  * `text_interface`
  This is a module handling all the user input. If this were a Model-View-Controller, this would be the View-Controller
  * `geo_utils` and `str_utils`
  modules containing helper functions for geographic and string tasks.

## module `gtfs`
Contains the `Agency`, `Route`, `Trip`, `StopTime`, `Service`, `Stop` and `ServiceException` structures
and functions to load these. For more information about these structures in the GTFS feed, please
refer to the official [GTFS docs](https://developers.google.com/transit/gtfs/reference/).

## module `network` - the model
The `network` module contains the `Network` structure, basically the model structure of the whole program.
It represents the whole transport network as a DAG (directed acyclic graph). The DAG is represented
with the `Node` structure in a vector (inflatable array), with each of the nodes having a list of edges
to the nodes reachable from it. Since it is not easy to represent the edges as pointers in Rust, we use the  
indices in the vector instead.

Each node in the DAG represents a location and time. The location can be either a trip, meaning the passenger
would be in a vehicle somewhere, or it can be a stop, meaning the passenger is waiting somewhere or that he has reached the
end of the journey.

To then explain how the DAG is structured, it is best to look at how it is created. The process is heavily
inspired by the one in KSP [here](http://ksp.mff.cuni.cz/h/ulohy/32/zadani3.html#task-32-3-6), so a more detailed
guide can be found there.

First, we go through all the StopTime structures provided in the GTFS feed and create all the transit nodes
associated with bus/tram/metro trips, creating the corresponding edges on every line. For each one of these StopTimes,
we also create an arrival node and a departure node at the stop where the StopTime is located. We then put
an edge from the transit node to the arrival node and an edge from the departure node to the transit node to
represent boarding and exiting vehicles.

After we create all the transit, departure and arrival nodes, we sort the arrival and departure nodes
at each stop by time and put edges in between them, so that one can actually transfer. These form the stop node
chains that are used for transfers between means of transport.

Then, we calculate physical distances between stops to add pedestrian transfers. We do this by first
dividing the map of Prague into squares of 500x500 metres (500m is going to be the largest pedestrian
distance we shall allow). We then take every square and check the distances in the square it belongs
to and all the neighbouring squares. If we find a stop that is closer than 500 metres, we add
a pedestrian connection.  

After we have calculated the pedestrian connections, we need to add these to the DAG. We do it
by taking every node belonging to a stop and checking whether there are transfers to neighbouring
stops. If there are, we calculate the time it takes to transfer and connect the node to the neighbouring
node which is after we arrive at the stop as pedestrians.

Since GTFS stops are not really stops as we think of them usually, but usually represent platforms etc.,
we also create stop groups. These are groups of stops that share the same main ID and represent the same transfer point 
("uzel" in Czech).

Now, when the user requests a connection lookup and provides us with the departure and destination stop names, as well
as the time of departure, we do this:
  * we go through all the stop groups and we find the stop group with a name that has the longest shared prefix with the requested name
  * we find the first nodes that are present at the stop group
  * we add the nodes to a minimal heap and run a dijkstra algorithm to find the shortest connection
  * we return the connection as a list of nodes the connection goes through

## module `text_interface` - the UI
This module contains the struct `TextInterface`, which is the main parser of input in the program,
and also a lot of helper methods. TextInterface uses the crate `rustyline` to provide a prompt.
It takes in lines from that prompt and then parses them into `Command` enums. These enums are then handled
and executed.

## module `geo_utils`
Contains functions for calculating pedestrian connections. Uses the Proj library.

## module `str_utils`
Currently contains just a function to calculate length of common string prefixes.