const AIRPORTS_JSON: &[u8] = include_bytes!("../res/airports.json");
const METAR_URL: &str = "https://www.ilmailusaa.fi/backend.php?{%22mode%22:%22metar%22,%22radius%22:%22100%22,%22points%22:[{%22_area%22:%221%22}]}";
const DATA_URL: &str = "https://data.vatsim.net/v3/vatsim-data.json";

use serde_json::Value;

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
                    let lat = airport.get("lat").unwrap().as_f64().unwrap();
                    let lon = airport.get("lon").unwrap().as_f64().unwrap();
                    let metar = airport.get("metar").unwrap().as_str().unwrap();
                    airports.push(Airport { icao: icao.to_string(), lat, lon, metar: metar.to_string() });
                }
            }
            return Ok(airports);
        }
        return Err("Failed to parse METAR data".to_string());
    }

    return Err(format!("Failed to fetch METAR data: {}", response.status()));
}

pub async fn fetch_metars() -> Result<Vec<String>, String> {
    //let efhk_metar = "EFHK 191720Z 24008KT 9999 FEW040 BKN060 02/M01 Q1010 NOSIG";
    fetch_metars_data().await.map(|airports| {
        airports.iter().map(|airport| {
            format!("{}: {}", airport.icao, airport.metar)
        }).collect()
    })
}
