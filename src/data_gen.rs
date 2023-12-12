use rand::prelude::*;
use regex::Regex;
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;
use crate::expression_parser::{parse_expression, Expression};

pub static FIELDS: [&str; 15] = ["_type", "_id", "key", "name", "fullName", "iata_airport_code", "type", "country", "latitude", "longitude", "location_id", "inEurope", "countryCode", "coreCountry", "distance"];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FakeData {
    pub _type: String,
    pub _id: u32,
    pub key: Option<String>,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub iata_airport_code: Option<String>,
    pub r#type: String,
    pub country: String,
    pub geo_position: GeoPosition,
    pub location_id: u32,
    #[serde(rename = "inEurope")]
    pub in_europe: bool,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "coreCountry")]
    pub core_country: bool,
    pub distance: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoPosition {
    pub latitude: String,
    pub longitude: String,
}

pub trait RandomGen {
    fn random<T: Rng + ?Sized>(rng: &mut T) -> Self;
}

impl RandomGen for FakeData {
    fn random<T: Rng + ?Sized>(rng: &mut T) -> Self {
        //array of format (country name, in europe, country code, core country)
        const COUNTRIES: [(&str, bool, &str, bool); 42] = [("Poland", true, "PL", true), ("Germany", true, "DE", true), ("France", true, "FR", true), ("Spain", true, "ES", true), ("Italy", true, "IT", true), ("United Kingdom", true, "GB", true), ("Netherlands", true, "NL", true), ("Belgium", true, "BE", true), ("Greece", true, "GR", true), ("Portugal", true, "PT", true), ("Sweden", true, "SE", true), ("Hungary", true, "HU", true), ("Austria", true, "AT", true), ("Czech Republic", true, "CZ", true), ("Ireland", true, "IE", true), ("Denmark", true, "DK", true), ("Finland", true, "FI", true), ("Norway", true, "NO", true), ("Romania", true, "RO", true), ("Turkey", false, "TR", true), ("Russia", false, "RU", true), ("Switzerland", false, "CH", true), ("Ukraine", false, "UA", true), ("Bulgaria", false, "BG", true), ("Serbia", false, "RS", true), ("Croatia", false, "HR", true), ("Iceland", false, "IS", true), ("Slovakia", false, "SK", true), ("Estonia", false, "EE", true), ("United States", false, "US", true), ("Canada", false, "CA", true), ("Mexico", false, "MX", true), ("Brazil", false, "BR", true), ("Australia", false, "AU", true), ("New Zealand", false, "NZ", true), ("China", false, "CN", true), ("India", false, "IN", true), ("Japan", false, "JP", true), ("South Korea", false, "KR", true), ("South Africa", false, "ZA", true), ("Egypt", false, "EG", true), ("Morocco", false, "MA", false)];
        const STREET_NAMES: [&str; 37] = ["Akacjowa", "Polna", "Kominiarska", "Kwiatowa", "Szkolna", "Kościelna", "Słoneczna", "Ogrodowa", "Topolowa", "Lipowa", "Brzozowa", "Kluczorska", "Klonowa", "Długa", "Krótka", "Kwiska", "Krucza", "Rolanda", "Koszykowa", "Garnizonowa", "Torpedowa", "Bojowników", "Kosynierów", "Marynarska", "Ekwadorska", "Zakopiańska", "Kasprowicza", "Kościuszki", "Słowackiego", "Kopernika", "Sienkiewicza", "Mickiewicza", "Kochanowskiego", "Reymonta", "Sobieskiego", "Piłsudskiego", "Kościelna"];

        let random_street_name = STREET_NAMES[rng.gen_range(0..STREET_NAMES.len())];
        let random_country = COUNTRIES[rng.gen_range(0..COUNTRIES.len())];

        FakeData {
            _type: String::from("Position"),
            _id: rng.next_u32(),
            key: Option::None,
            name: String::from(random_street_name),
            full_name: String::from(random_street_name)+", "+&random_country.0,
            iata_airport_code: Option::None,
            r#type: String::from("location"),
            country: String::from(random_country.0),
            geo_position: GeoPosition::random(rng),
            location_id: rng.next_u32(),
            in_europe: random_country.1,
            country_code: String::from(random_country.2),
            core_country: random_country.3,
            distance: Option::None
        }
    }
}

impl RandomGen for GeoPosition {
    fn random<T: Rng + ?Sized>(rng: &mut T) -> Self {
        GeoPosition {
            latitude: format!{"{:.7}", rng.gen_range(-90.0..90.0)},
            longitude: format!{"{:.7}", rng.gen_range(-180.0..180.0)}
        }
    }
}

impl FakeData {
    fn get_filtered_indexmap(&self, fields: &Vec<&str>) -> IndexMap<String, Expression> {
        let mut map = IndexMap::new();
        for field in fields { match *field {
                "_type" => map.insert(String::from("_type"), Expression::String(self._type.clone())),
                "_id" => map.insert(String::from("_id"), Expression::Number(self._id as i64)),
                "key" => map.insert(String::from("key"), Expression::String(self.key.clone().unwrap_or(String::from("null")))),
                "name" => map.insert(String::from("name"), Expression::String(self.name.clone())),
                "fullName" => map.insert(String::from("fullName"), Expression::String(self.full_name.clone())),
                "iata_airport_code" => map.insert(String::from("iata_airport_code"), Expression::String(self.iata_airport_code.clone().unwrap_or(String::from("null")))),
                "type" => map.insert(String::from("type"), Expression::String(self.r#type.clone())),
                "country" => map.insert(String::from("country"), Expression::String(self.country.clone())),
                "latitude" => map.insert(String::from("latitude"), Expression::Float(self.geo_position.latitude.parse().unwrap())),
                "longitude" => map.insert(String::from("longitude"), Expression::Float(self.geo_position.longitude.parse().unwrap())),
                "location_id" => map.insert(String::from("location_id"), Expression::Number(self.location_id as i64)),
                "inEurope" => map.insert(String::from("inEurope"), Expression::String(self.in_europe.to_string())),
                "countryCode" => map.insert(String::from("countryCode"), Expression::String(self.country_code.clone())),
                "coreCountry" => map.insert(String::from("coreCountry"), Expression::String(self.core_country.to_string())),
                "distance" => map.insert(String::from("distance"), Expression::Float(self.distance.as_ref().unwrap_or(&0.0).to_string().parse().unwrap())),
                _ => map.insert(String::from("None"), Expression::String(String::from("None")))
            };
        }
        return map;
    }

    pub fn to_computed_vec(&self, fields: &Vec<String>) -> Result<Vec<String>, String> {
        let used_fields: Vec<&str> = FIELDS.clone().into_iter().filter(|x| fields.iter().any(|y| {
            let re = Regex::new(&format!(r"\b{}\b", x)).unwrap();
            re.is_match(y)
        })).collect();
        let map = self.get_filtered_indexmap(&used_fields);

        fields.iter().map(|field| {
            let parsed = parse_expression(field)?;
            let result = parsed.eval(&map)?;
            Ok(result.to_string())
        }).collect()
    }
}