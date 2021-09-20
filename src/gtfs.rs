use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use serde::{de, de::Unexpected, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Agency {
    pub agency_id: String,
    pub agency_name: String,
    pub agency_url: String,
    pub agency_timezone: String,
    pub agency_lang: String,
    pub agency_phone: String,
}

#[derive(Debug, Deserialize)]
pub struct Route {
    pub route_id: String,
    pub agency_id: String,
    pub route_short_name: String,
    pub route_long_name: String,
    pub route_type: u32,
    pub route_url: Option<String>,
    pub route_color: Option<String>,
    pub route_text_color: Option<String>,
    #[serde(deserialize_with = "bool_from_int")]
    pub is_night: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Trip {
    pub route_id: String,
    pub service_id: String,
    pub trip_id: String,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: u8,
    pub block_id: Option<String>,
    pub shape_id: Option<String>,
    pub wheelchair_accessible: Option<u8>,
    pub bikes_allowed: Option<u8>,
    pub exceptional: Option<u8>,
    pub trip_operation_type: Option<u8>,
    #[serde(default = "Vec::new", skip_deserializing)]
    pub stop_times: Vec<StopTime>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StopTime {
    pub trip_id: String,
    #[serde(deserialize_with = "deserialize_time")]
    // time of the day in seconds
    pub arrival_time: u32,
    #[serde(deserialize_with = "deserialize_time")]
    // time of the day in seconds
    pub departure_time: u32,
    pub stop_id: String,
    pub stop_sequence: u32,
    pub stop_headsign: Option<String>,
    pub pickup_type: u8,
    pub drop_off_type: u8,
    pub shape_dist_travelled: Option<f32>,
}

fn deserialize_ymd<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(NaiveDate::parse_from_str(&s, "%Y%m%d").unwrap())
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let hms: Vec<u32> = s.split(":").map(|x| x.parse::<u32>().unwrap()).collect();
    return Ok(3600 * hms[0] + 60 * hms[1] + hms[2]);
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub service_id: String,
    #[serde(deserialize_with = "bool_from_int")]
    pub monday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub tuesday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub wednesday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub thursday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub friday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub saturday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub sunday: bool,
    #[serde(deserialize_with = "deserialize_ymd")]
    pub start_date: NaiveDate,
    #[serde(deserialize_with = "deserialize_ymd")]
    pub end_date: NaiveDate,
    #[serde(default = "Vec::new", skip_deserializing)]
    pub exceptions: Vec<ServiceException>,
}

/// A structure describing a stop.
#[derive(Debug, Deserialize, Clone)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f32,
    pub stop_lon: f32,
    pub zone_id: String,
    pub stop_url: Option<String>,
    pub location_type: u8,
    pub parent_station: Option<String>,
    pub wheelchair_boarding: Option<i32>,
    pub level_id: Option<String>,
    pub platform_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceException {
    pub service_id: String,
    #[serde(deserialize_with = "deserialize_ymd")]
    pub date: NaiveDate,
    pub exception_type: u8,
}

/// Loads the contents of stops.txt
/// # Arguments
/// * path - the path to the directory stops.txt is located in
pub fn load_stops(path: &Path) -> HashMap<String, Stop> {
    let mut stops = HashMap::new();
    let mut file_path_buf = path.to_path_buf();
    file_path_buf.push(Path::new("stops.txt"));
    let file = File::open(file_path_buf.as_path()).unwrap(); // No need for error handling, if this fails, we want to panic
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: Stop = result.unwrap();
        stops.insert(record.stop_id.clone(), record);
    }
    return stops;
}

#[test]
fn test_stop_loading() {
    let stops = load_stops(Path::new("test_data/"));
    assert_eq!(1, stops.len());
    let stop = stops.get("U50S1").unwrap();
    assert_eq!(stop.stop_id, "U50S1");
    assert_eq!(stop.stop_name, "Budějovická");
    assert_eq!(stop.stop_lat, 50.04441);
    assert_eq!(stop.stop_lon, 14.44879);
    assert_eq!(stop.zone_id, "P");
    assert_eq!(stop.stop_url, None);
    assert_eq!(stop.location_type, 1);
    assert_eq!(stop.parent_station, None);
    assert_eq!(stop.wheelchair_boarding, Some(1));
    assert_eq!(stop.level_id, None);
    assert_eq!(stop.platform_code, None);
}

/// Loads the contents of routes.txt
/// # Arguments
/// * path - the path to the directory routes.txt is located in
pub fn load_routes(path: &Path) -> HashMap<String, Route> {
    let mut routes = HashMap::new();
    let mut file_path_buf = path.to_path_buf();
    file_path_buf.push(Path::new("routes.txt"));
    let file = File::open(file_path_buf.as_path()).unwrap(); // No need for error handling, if this fails, we want to panic
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: Route = result.unwrap();
        routes.insert(record.route_id.clone(), record);
    }
    return routes;
}

#[test]
fn test_route_loading() {
    let routes = load_routes(Path::new("test_data/"));
    assert_eq!(1, routes.len());
    let route = routes.get("L991").unwrap();
    assert_eq!(route.route_id, "L991");
    assert_eq!(route.agency_id, "99");
    assert_eq!(route.route_short_name, "A");
    assert_eq!(
        route.route_long_name,
        "Nemocnice Motol - Petřiny - Skalka - Depo Hostivař"
    );
    assert_eq!(route.route_type, 1);
    assert_eq!(
        route.route_url,
        Some(String::from("https://pid.cz/linka/A"))
    );
    assert_eq!(route.route_color, Some(String::from("00A562")));
    assert_eq!(route.route_text_color, Some(String::from("FFFFFF")));
    assert_eq!(route.is_night, false);
}

/// Loads the contents of trips.txt
/// # Arguments
/// * path - the path to the directory trips.txt is located in
pub fn load_trips(path: &Path) -> HashMap<String, Trip> {
    let mut trips = HashMap::new();
    let mut file_path_buf = path.to_path_buf();
    file_path_buf.push(Path::new("trips.txt"));
    let file = File::open(file_path_buf.as_path()).unwrap(); // No need for error handling, if this fails, we want to panic
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: Trip = result.unwrap();
        trips.insert(record.trip_id.clone(), record);
    }
    return trips;
}

#[test]
fn test_trip_loading() {
    let trips = load_trips(Path::new("test_data/"));
    assert_eq!(trips.len(), 1);
    let trip = trips.get("991_1411_191224").unwrap();
    assert_eq!(trip.route_id, "L991");
    assert_eq!(trip.service_id, "0000010-1");
    assert_eq!(trip.trip_id, "991_1411_191224");
    assert_eq!(trip.trip_headsign, Some(String::from("Nemocnice Motol")));
    assert_eq!(trip.trip_short_name, None);
    assert_eq!(trip.direction_id, 0);
    assert_eq!(trip.block_id, None);
    assert_eq!(trip.shape_id, Some(String::from("L991V1")));
    assert_eq!(trip.wheelchair_accessible, Some(1));
    assert_eq!(trip.bikes_allowed, Some(1));
    assert_eq!(trip.exceptional, Some(0));
    assert_eq!(trip.trip_operation_type, Some(1));
}

/// Loads the contents of services.txt and service_dates.txt
/// # Arguments
/// * path - the path to the directory the files are located in
pub fn load_services(path: &Path) -> HashMap<String, Service> {
    let mut services = HashMap::new();
    let mut file_path_buf = path.to_path_buf();
    file_path_buf.push(Path::new("calendar.txt"));
    let file = File::open(file_path_buf.as_path()).unwrap(); // No need for error handling, if this fails, we want to panic
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: Service = result.unwrap();
        services.insert(record.service_id.clone(), record);
    }
    return services;
}

#[test]
fn test_service_loading() {
    let services = load_services(Path::new("test_data/"));
    assert_eq!(services.len(), 1);
    let service = services.get("0000010-1").unwrap();
    assert_eq!(service.monday, false);
    assert_eq!(service.tuesday, false);
    assert_eq!(service.wednesday, false);
    assert_eq!(service.thursday, false);
    assert_eq!(service.friday, false);
    assert_eq!(service.saturday, true);
    assert_eq!(service.sunday, false);
    assert_eq!(service.start_date, NaiveDate::from_ymd(2020, 1, 25));
    assert_eq!(service.end_date, NaiveDate::from_ymd(2020, 2, 7))
}

/// Loads service exceptions from calendar_dates.txt and adds them to the HashMap
/// # Arguments
/// * path - the path to the gtfs directory
/// * services - loaded contents of calendar.txt
pub fn load_service_exceptions(path: &Path, services: &mut HashMap<String, Service>) {
    let mut file_path_buf = path.to_path_buf();
    file_path_buf.push(Path::new("calendar_dates.txt"));
    let file = File::open(file_path_buf.as_path()).unwrap(); // No need for error handling, if this fails, we want to panic
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: ServiceException = result.unwrap();
        services
            .get_mut(&record.service_id)
            .unwrap()
            .exceptions
            .push(record);
    }
}

// FIXME StopTime loading is slow as hell. Probably it is due to the amout of StopTimes.
// Perhaps some kind of buffering could help?
pub fn load_stop_times(path: &Path, trips: &mut HashMap<String, Trip>) {
    let mut file_path_buf = path.to_path_buf();
    file_path_buf.push(Path::new("stop_times.txt"));
    let file = File::open(file_path_buf.as_path()).unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let stop_time: StopTime = result.unwrap();
        trips
            .get_mut(&stop_time.trip_id)
            .unwrap()
            .stop_times
            .push(stop_time);
    }
    for trip in trips.values_mut() {
        trip.stop_times
            .sort_by(|a, b| a.stop_sequence.cmp(&b.stop_sequence));
    }
}