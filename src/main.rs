/// Program that collects data from AISHub.net
/// 
/// Author: G0rocks
/// Date created: 2025-10-20

// Crate imports
use serde::Deserialize; // For deserializing JSON
use serde_json;      // For parsing JSON
use csv;             // For reading CSV files
use std::fs;        // For file system operations

fn main() {
    println!("Running aishub_data_collector");

    // Infinite loop to collect data periodically

        // Get settings from settings file
        let settings = get_settings();
        println!("Settings: {:?}", settings);

        // Get list of ships to monitor
        // let ships = get_list_of_ships();
        // println!("Ships to monitor: {:?}", ships);

        // Make URL
        // let url = make_aishub_url("my_username", Some("csv"), Some("full"), Some("0"), Some(-118.0), None, None, None, None, None, Some(60));
        // println!("AISHub URL: {}", url);

        // Collect data using API

        // Store data in database

        // Wait until next interval
}

// Structs
// --------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
struct Settings {
    api_key: String,
    update_interval: u32,
}


// Functions
// --------------------------------------------------------------------------------------

/// Gets settings from settings file
/// API key, loop interval (in minutes)
fn get_settings() -> Settings {
    // Parse settings.json file
    let contents = fs::read_to_string("settings.json")?;
    let settings: Settings = serde_json::from_str(&contents)?;

    // Return settings
    return settings;
}

/// Gets list of ships to monitor from ships.csv file
/// Returns a tuple of two vectors: (mmsi_numbers, imo_numbers)
fn get_list_of_ships() -> (Vec<String>, Vec<String>) {
    let mut mmsi: Vec<String> = Vec::new();
    let mut imo: Vec<String> = Vec::new();

    // Read ships.csv file
    let mut rdr = csv::Reader::from_path("ships.csv")?;
    for result in rdr.records() {
        let record = result?;
        mmsi.push(record[0].to_string());
        imo.push(record[1].to_string());
    }

    // Return tuple of vectors
    return (mmsi, imo);
}

/// Makes the URL for the AISHub API request
/// Based on https://www.aishub.net/api
fn make_aishub_url(username: &str, format: Option<&str>, output: Option<&str>, compress: Option<&str>, latmin: Option<f64>, latmax: Option<f64>, lonmin: Option<f64>, lonmax: Option<f64>, mmsi: Option<&str>, imo: Option<&str>, interval: Option<u32>) -> String {
    let mut url = format!("https://data.aishub.net/ws.php?username={}", username);

    // If parameters are provided, add them to the URL
    match format {
        Some(value) => url.push_str(&format!("&format={}", value)),
        None => {}
    }
    match output {
        Some(value) => url.push_str(&format!("&output={}", value)),
        None => {}
    }
    match compress {
        Some(value) => url.push_str(&format!("&compress={}", value)),
        None => {}
    }
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