/// Program that collects data from AISHub.net
/// 
/// Author: G0rocks
/// Date created: 2025-10-20

// Crate imports
use serde::Deserialize; // For deserializing JSON
use serde_json;      // For parsing JSON
use csv;             // For reading CSV files
use std::fs;        // For file system operations
use reqwest;      // For making HTTP requests
use time;     // For handling time

fn main() {
    // Startup message
    println!("Starting AISHub Data Collector... Press ctrl+C to stop.");
    // Init start time
    let start_time = time::UtcDateTime::now();

    // Init default update_interval (in minutes)
    let mut update_interval: u32 = 1;

    // Infinite loop to collect data periodically
    loop {
        // Print status message
        println!("Collecting data from AISHub at {:?} for {}", time::UtcDateTime::now(), time::UtcDateTime::now() - start_time);
        // Get settings from settings file
        let settings = get_settings();
        // Update update_interval from settings
        update_interval = settings.update_interval;

        // Get list of ships to monitor
        let (imo_nums, mmsi_nums) = get_list_of_ships();
        let imo = vec_to_comma_separated_string(&imo_nums);
        let mmsi = vec_to_comma_separated_string(&mmsi_nums);

        // Make URL
        let url = make_aishub_url(settings.api_key.as_str(), settings.data_value_format, settings.output_format.as_str(), settings.compression, None, None, None, None, mmsi.as_deref(), imo.as_deref(), None);
        println!("AISHub URL: {}", url);

        // Collect data using API
        let data = get_data_from_aishub_api(url);

        // Store data in database
        save_data(data);

        // Wait until next interval
        std::thread::sleep(std::time::Duration::from_secs((update_interval * 60) as u64));
    }
}

// Structs
// --------------------------------------------------------------------------------------
/// The user settings the program needs to make the API requests
#[derive(Debug, Deserialize)]
struct Settings {
    api_key: String,
    update_interval: u32,
    data_value_format: u8,
    output_format: String,
    compression: u8,
    min_lat: Option<f32>,
    max_lat: Option<f32>,
    min_lon: Option<f32>,
    max_lon: Option<f32>,
    max_age: Option<u64>
}

/// The ship info received from AISHub API
/// Based on the explanation of data values at https://www.aishub.net/api
/// Fields should always be in alphabetical order
#[derive(Debug)]
struct VesselInfo {
    /// Dimension to Bow (meters). If unknown, value is zero
    a:  u64,
    /// Dimension to Stern (meters). If unknown, value is zero
    b:  u64,
    /// Dimension to Port (meters). If unknown, value is zero
    c:  u64,
    /// vessel’s callsign. If unknown, value is empty string
    callsign:   String,
    /// Course Over Ground AIS format – in 1/10 degrees i.e. degrees multiplied by 10. COG=3600 means “not available” Human readable format – degrees. COG=360.0 means “not available” 
    cog:    f64,
    /// Dimension to Starboard (meters). If unknown, value is zero
    d:  u64,
    /// vessel’s destination. If unknown, value is empty string
    dest:   String,
    /// AIS format – in 1/10 meters i.e. draught multiplied by 10. Human readable format – meters. If unknown, value is zero
    draught:    u64,
    /// positioning device type. If unknown, value is empty string
    device:    String,
    /// Estimated Time of Arrival. AIS format (see here link broken at 2025-10-22). Human readable format – UTC date/time. If unknown, value is zero
    eta:    u64,
    /// current heading of the AIS vessel at the time of the last message value in degrees, HEADING=511 means “not available”
    heading:    u64,
    /// IMO ship identification number. If unknown, value is zero
    imo:    u64,
    /// geographical latitude AIS format – in 1/10000 minute i.e. degrees multiplied by 600000 Human readable format – degrees. If unknown, value is empty string
    latitude:   String,
    /// geographical longitude AIS format – in 1/10000 minute i.e. degrees multiplied by 600000 Human readable format – degrees. If unknown, value is empty string
    longitude:  String,
    /// Maritime Mobile Service Identity. If unknown, value is zero
    mmsi:   u64,
    /// vessel’s name (max.20 chars). If unknown, value is empty string
    name:   String,
    /// Navigational Status. If unknown, value is empty string
    navstat:    String,
    /// (AIS format only) – Position Accuracy 0 – low accuracy 1 – high accuracy. If unknown, low accuracy is assumed and value is zero
    pac:   u8,
    /// (AIS format only) - Rate of Turn. If unknown, value is empty string
    rot:    String,
    /// Speed Over Ground AIS format – in 1/10 knots i.e. knots multiplied by 10. SOG=1024 means “not available” Human readable format – knots. SOG=102.4 means “not available” 
    sog:    u64,
    ///  	data timestamp AIS format – unix timestamp Human readable format – UTC. If unknown, value is zero
    timestamp: u64,
    /// vessel’s type. If unknown, value is zero
    vessel_type:   u64,
}

impl VesselInfo {
    /// Creates a new VesselInfo struct with default AIS format values indicating unknown data
    fn new() -> VesselInfo {
        VesselInfo {
            a: 0,
            b: 0,
            c: 0,
            callsign: String::new(),
            cog: 3600.0,
            d: 0,
            dest: String::new(),
            draught: 0,
            device: String::new(),
            eta: 0,
            heading: 511,
            imo: 0,
            latitude: String::new(),
            longitude: String::new(),
            mmsi: 0,
            name: String::new(),
            navstat: String::new(),
            pac: 0,
            rot: String::new(),
            sog: 1024,
            timestamp: 0,
            vessel_type: 0,
        }
    }
}



// Functions
// --------------------------------------------------------------------------------------

/// Gets settings from settings file
/// API key, loop interval (in minutes)
fn get_settings() -> Settings {
    // Parse settings.json file
    let contents = match fs::read_to_string("settings.json") {
        Ok(c) => c,
        Err(e) => {
            panic!("Error reading settings.json file: {}", e);
        }
    };
    let settings: Settings = serde_json::from_str(&contents).expect("Error parsing settings.json file");

    // Return settings
    return settings;
}

/// Gets list of ships to monitor from ships.csv file
/// Returns a tuple of two vectors: (mmsi_numbers, imo_numbers)
/// Prioritizes IMO numbers over MMSI numbers so if both are provided, IMO is used
fn get_list_of_ships() -> (Vec<String>, Vec<String>) {
    let mut mmsi: Vec<String> = Vec::new();
    let mut imo: Vec<String> = Vec::new();

    // Read ships.csv file
    let mut rdr = match csv::Reader::from_path("ships.csv") {
        Ok(r) => r,
        Err(e) => {
            panic!("Error reading ships.csv file: {}", e);
        }
    };

    // For each entry, if MMSI or IMO is provided, add to respective vector
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                panic!("Error reading record from ships.csv file: {}", e);
            }
        };
        // If imo number is provided, add to imo vector
        if !record[0].is_empty() {
            imo.push(record[0].to_string());
            continue;
        }
        if record[1].is_empty() {
            continue; // Skip if both are empty
        }
        // Add mmsi number
        mmsi.push(record[1].to_string());
    }

    // Return tuple of vectors
    return (imo, mmsi);
}


/// Takes in a vector of strings and returns a single string with commas between the values
/// E.g. ["123", "456", "789"] -> "123,456,789"
fn vec_to_comma_separated_string(vec: &Vec<String>) -> Option<String> {
    // Return None if vector is empty
    if vec.is_empty() {
        return None;
    }

    // Loop through vector and build string
    let mut result = String::new();
    for (i, value) in vec.iter().enumerate() {
        result.push_str(value);
        if i < vec.len() - 1 {
            result.push_str(","); // Add comma if not the last value
        }
    }

    return Some(result);
}

/// Makes the URL for the AISHub API request
/// Based on https://www.aishub.net/api
fn make_aishub_url(api_key: &str, data_value_format: u8, output_format: &str, compression: u8, latmin: Option<f64>, latmax: Option<f64>, lonmin: Option<f64>, lonmax: Option<f64>, mmsi: Option<&str>, imo: Option<&str>, interval: Option<u32>) -> String {
    let mut url = format!("https://data.aishub.net/ws.php?username={}&format={}&output={}&compress={}", api_key, data_value_format, output_format, compression);

    // Add optional parameters
    match latmin {
        Some(value) => url.push_str(&format!("&latmin={}", value)),
        None => {}
    }
    match latmax {
        Some(value) => url.push_str(&format!("&latmax={}", value)),
        None => {}
    }
    match lonmin {
        Some(value) => url.push_str(&format!("&lonmin={}", value)),
        None => {}
    }
    match lonmax {
        Some(value) => url.push_str(&format!("&lonmax={}", value)),
        None => {}
    }
    match mmsi {
        Some(value) => url.push_str(&format!("&mmsi={}", value)),
        None => {}
    }
    match imo {
        Some(value) => url.push_str(&format!("&imo={}", value)),
        None => {}
    }
    match interval {
        Some(value) => url.push_str(&format!("&interval={}", value)),
        None => {}
    }

    // Return URL
    return url;
}

/// Function that fetches data from AISHub API given a URL
/// Assumes only 1 data point is returned per ship
fn get_data_from_aishub_api(url: String) -> Vec<VesselInfo> {
    // Get the result of the request
    let body = match reqwest::blocking::get(url) {
        Ok(response) => {
            match response.text() {
                Ok(text) => text,
                Err(e) => {
                    panic!("Error reading response text: {}", e);
                }
            }
        },
        Err(e) => {
            panic!("Error making request to AISHub API: {}", e);
        }
    };

    // If too frequent requests are made, stop running
    if body == "Too frequent requests!" {
        panic!("Too frequent requests made to AISHub API. Stopping program. Please check your update interval and make sure it is big enough.");
    }

    // Get CSV reader from body
    let mut rdr = csv::Reader::from_reader(body.as_bytes());

    // Get order of headers
    let headers = rdr.headers().unwrap().clone();
    let header_order = get_header_order(&headers);

    println!("Headers: {:?}", headers);
    println!("Header order: {:?}", header_order);

    // Init empty vector to hold data
    let mut data: Vec<VesselInfo> = Vec::new();

    // Loop through each line of the response body, append each data point to data vector
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                panic!("Error reading record from CSV response: {}", e);
            }
        };

        println!("Record: {:?}", record);
        
        // Create default VesselInfo struct
        let mut vessel_info = VesselInfo::new();

        // Fill in values that exist based on header order
        match header_order[0] {
            Some(index) => vessel_info.a = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[1] {
            Some(index) => vessel_info.b = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[2] {
            Some(index) => vessel_info.c = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[3] {
            Some(index) => vessel_info.callsign = record[index].to_string(),
            None => {}
        }
        match header_order[4] {
            Some(index) => vessel_info.cog = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[5] {
            Some(index) => vessel_info.d = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[6] {
            Some(index) => vessel_info.dest = record[index].to_string(),
            None => {}
        }
        match header_order[7] {
            Some(index) => vessel_info.draught = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[8] {
            Some(index) => vessel_info.device = record[index].to_string(),
            None => {}
        }
        match header_order[9] {
            Some(index) => vessel_info.eta = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[10] {
            Some(index) => vessel_info.heading = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[11] {
            Some(index) => vessel_info.imo = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[12] {
            Some(index) => vessel_info.latitude = record[index].to_string(),
            None => {}
        }
        match header_order[13] {
            Some(index) => vessel_info.longitude = record[index].to_string(),
            None => {}
        }
        match header_order[14] {
            Some(index) => vessel_info.mmsi = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[15] {
            Some(index) => vessel_info.name = record[index].to_string(),
            None => {}
        }
        match header_order[16] {
            Some(index) => vessel_info.navstat = record[index].to_string(),
            None => {}
        }
        match header_order[17] {
            Some(index) => vessel_info.pac = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[18] {
            Some(index) => vessel_info.rot = record[index].to_string(),
            None => {}
        }
        match header_order[19] {
            Some(index) => vessel_info.sog = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[20] {
            Some(index) => vessel_info.timestamp = record[index].parse().unwrap(),
            None => {}
        }
        match header_order[21] {
            Some(index) => vessel_info.vessel_type = record[index].parse().unwrap(),
            None => {}
        }

        // Append to data vector
        data.push(vessel_info);
    }

    // Return the data vector
    return data;
}


/// Gets the order of headers in the CSV response
/// Returns a vector where the first value is the index of the first value in the VesselInfo struct, second value is the index of the second value, etc.
/// Based on the VesselInfo struct definition (alphabetical order) and https://www.aishub.net/api
fn get_header_order(headers: &csv::StringRecord) -> Vec<Option<usize>> {
    // Init vector to hold order
    let mut order: Vec<Option<usize>> = vec![None; 22];

    // Loop through headers and get index of each value
    for (i, header) in headers.iter().enumerate() {
        match header {
            "A" =>              order[0] = Some(i),
            "B" =>              order[1] = Some(i),
            "C" =>              order[2] = Some(i),
            "CALLSIGN" =>       order[3] = Some(i),
            "COG" =>            order[4] = Some(i),
            "D" =>              order[5] = Some(i),
            "DEST" =>           order[6] = Some(i),
            "DEVICE" =>         order[7] = Some(i),
            "DRAUGHT" =>        order[8] = Some(i),
            "ETA" =>            order[9] = Some(i),
            "HEADING" =>        order[10] = Some(i),
            "IMO" =>            order[11] = Some(i),
            "LATITUDE" =>       order[12] = Some(i),
            "LONGITUDE" =>      order[13] = Some(i),
            "MMSI" =>           order[14] = Some(i),
            "NAME" =>           order[15] = Some(i),
            "NAVSTAT" =>        order[16] = Some(i),
            "PAC" =>            order[17] = Some(i),
            "ROT" =>            order[18] = Some(i),
            "SOG" =>            order[19] = Some(i),
            "TSTAMP" =>         order[20] = Some(i),    // Timestamp header is "TSTAMP"
            "TYPE" =>           order[21] = Some(i),    // Vessel type header is "TYPE"
            _ => {panic!("Unknown header in CSV response: {}", header);}
        }
    }

    // Return order vector
    return order;
}

/// Function that saves the data to the database
/// If the files don't exist, creates them
/// If the files already exist, appends to them
fn save_data(data: Vec<VesselInfo>) {
    // Placeholder function to save data to database
    println!("Saving data to database: {:?}", data);
        //let response = reqwest::blocking::get(url)?;
        //let content = response.bytes()?;
//
        //let mut file = File::create(filename)?;
        //file.write_all(&content)?;
    //
        //println!("Saved CSV to {}", filename);
        //Ok(())

}