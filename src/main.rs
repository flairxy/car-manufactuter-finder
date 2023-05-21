#![deny(clippy::all)]

use std::env;

use reqwest::Client;

const API_URL: &str = "https://vpic.nhtsa.dot.gov/api/vehicles/getallmanufacturers?format=json";

struct Manufacturer<'a> {
    name: Option<&'a str>,
    common_name: Option<&'a str>,
    country: Option<&'a str>,
}

trait Contains {
    fn has(&self, needle: &str) -> bool;
}

impl<'a> Contains for Manufacturer<'a> {
    fn has(&self, needle: &str) -> bool {
        self.name.unwrap_or_default().contains(needle)
            || self.common_name.unwrap_or_default().contains(needle)
            || self.country.unwrap_or_default().contains(needle)
    }
}
impl<'a> Manufacturer<'a> {
    fn description(&self) -> String {
        let name = self.name.unwrap_or_default();
        let country = self.country.unwrap_or_default();
        let common_name = self.common_name.unwrap_or_default();
        format!("\tName: {name} \n\tCommon Name: {common_name} \n\tCountry: {country}")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} search term", args[0]);
        return Ok(());
    }
    let keyword = &args[1];
    let client = Client::new();
    let res = client
        .get(API_URL)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await.unwrap();

    //"Count": 10755,
    // "Message": "Response returned successfully",
    // "SearchCriteria": null,
    // "Results": [
    // {
    // "Make_ID": 11897,

     //since our incoming api json has other keys as shown above, we want to specify using the Results as key
    let manufacturer_json = res
        .as_object()
        .unwrap()
        .iter()
        .find(|(key, _)| key == &"Results")
        .unwrap()
        .1
        .as_array()
        .unwrap()
        .iter();

    let manufacturers = manufacturer_json.map(|data| {
        data.as_object().unwrap();
        let country = data.get("Country").unwrap().as_str();
        let common_name = data.get("Mfr_CommonName").unwrap().as_str();
        let name = data.get("Mfr_Name").unwrap().as_str();
        Manufacturer { name, common_name, country }
    });

    let found_manufacturers = 
        manufacturers.filter(|manufacturer| manufacturer.has(keyword)).collect::<Vec<_>>();
    
    if found_manufacturers.is_empty() {
        Err("No manufacturer found".into())
    }else{
        println!("Found {}", found_manufacturers.len());
        for (index, man) in found_manufacturers.iter().enumerate(){
            println!("Manufacter #{}", index + 1);
            println!("{}", man.description());
        }
        Ok(())
    }
}
