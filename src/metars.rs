const METAR_URL: &str = "https://www.ilmailusaa.fi/backend.php?{%22mode%22:%22metar%22,%22radius%22:%22100%22,%22points%22:[{%22_area%22:%221%22}]}";
const DATA_URL: &str = "https://data.vatsim.net/v3/vatsim-data.json";

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::f64::consts::PI;

#[derive(Debug)]
struct Airport {
    icao: String,
    lat: f64,
    lon: f64,
    metar: String,
}

async fn fetch_metars_data() -> Result<Vec<Airport>, String> {
    let response = reqwest::get(METAR_URL).await.or_else(|e| {
        return Err(format!("Failed to fetch METAR data: {}", e));
    })?;

    if response.status().is_success() {
        let body = response.text().await.or_else(|e| {
            return Err(format!("Failed to read response body: {}", e));
        })?;

        let raw: Value = serde_json::from_str(&body).or_else(|e| {
            return Err(format!("Failed to parse METAR data: {}", e));
        })?;

        if let Value::Object(map) = raw {
            let mut airports = Vec::new();
            for (key, value) in map {
                if let Value::Object(airport) = value {
                    let icao = key;
                    let lat: f64 = airport
                        .get("lat")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let lon: f64 = airport
                        .get("lon")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let metar = airport.get("p1").unwrap().as_str().unwrap();
                    airports.push(Airport {
                        icao: icao.strip_prefix("METAR").unwrap().to_string(),
                        lat,
                        lon,
                        metar: metar.to_string(),
                    });
                }
            }
            return Ok(airports);
        }
        return Err("Failed to parse METAR data".to_string());
    }

    return Err(format!("Failed to fetch METAR data: {}", response.status()));
}

#[derive(Serialize, Deserialize, Debug)]
struct VatsimData {
    pilots: Vec<VatsimPilot>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VatsimPilot {
    callsign: String,
    cid: u32,
    name: String,
    latitude: f64,
    longitude: f64,
    altitude: i32,
    groundspeed: i32,
    heading: i32,
    flight_plan: Option<VatsimFlightPlan>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VatsimFlightPlan {
    aircraft_short: String,
    departure: String,
    arrival: String,
    alternate: String,
    flight_rules: String,
    remarks: String,
    route: String,
}

async fn fetch_vatsim_data() -> Result<VatsimData, String> {
    let response = reqwest::get(DATA_URL).await.or_else(|e| {
        return Err(format!("Failed to fetch VATSIM data: {}", e));
    })?;

    if response.status().is_success() {
        let body = response.text().await.or_else(|e| {
            return Err(format!("Failed to read response body: {}", e));
        })?;

        let data: VatsimData = serde_json::from_str(&body).or_else(|e| {
            return Err(format!("Failed to parse VATSIM data: {}", e));
        })?;

        return Ok(data);
    }

    return Err(format!(
        "Failed to fetch VATSIM data: {}",
        response.status()
    ));
}

pub async fn fetch_metars() -> Result<Vec<String>, String> {
    let airports = fetch_metars_data().await.or_else(|e| {
        return Err(format!("Failed to fetch METAR data: {}", e));
    })?;
    let data = fetch_vatsim_data().await.or_else(|e| {
        return Err(format!("Failed to fetch VATSIM data: {}", e));
    })?;

    let to_or_from_efin: Vec<_> = data
        .pilots
        .iter()
        .filter(|pilot| match &pilot.flight_plan {
            Some(flight_plan) => {
                flight_plan.departure.starts_with("EF") || flight_plan.arrival.starts_with("EF")
            }
            None => false,
        })
        .collect();

    let within_300nm_of_dest_or_dep: Vec<_> = to_or_from_efin
        .iter()
        .filter(|pilot| {
            let destination_or_arrival = airports.iter().find(|airport| match &pilot.flight_plan {
                Some(flight_plan) => {
                    flight_plan.arrival == airport.icao || flight_plan.departure == airport.icao
                }
                None => false,
            });

            if destination_or_arrival.is_none() {
                return false;
            } else {
                let destination = destination_or_arrival.unwrap();
                let distance = {
                    const R: f64 = 6371.0;
                    let x = (destination.lon - pilot.longitude) * PI / 180.0
                        * ((destination.lat + pilot.latitude) * PI / 180.0).cos();
                    let y = (destination.lat - pilot.latitude) * PI / 180.0;
                    R * (x * x + y * y).sqrt()
                };
                distance < 300.0
            }
        })
        .collect();

    Ok(airports
        .iter()
        .filter(|airport| {
            within_300nm_of_dest_or_dep
                .iter()
                .any(|pilot| match &pilot.flight_plan {
                    Some(flight_plan) => {
                        flight_plan.arrival == airport.icao || flight_plan.departure == airport.icao
                    }
                    None => false,
                })
        })
        .map(|airport| airport.metar.clone())
        .collect())
}
