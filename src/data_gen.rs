use rand::prelude::*;
use regex::Regex;
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

pub static FIELDS: [&str; 15] = ["_type", "_id", "key", "name", "fullName", "iata_airport_code", "type", "country", "latitude", "longtitude", "location_id", "inEurope", "countryCode", "coreCountry", "distance"];

#[derive(Debug, Clone)]
pub enum FieldType {
    String(String),
    U32(u32),
    StringOption(Option<String>),
    Bool(bool),
    F64Option(Option<f64>),
}

impl FieldType {
    pub fn to_string(&self) -> String {
        match self {
            FieldType::String(s) => s.clone(),
            FieldType::U32(u) => u.to_string(),
            FieldType::StringOption(s) => s.clone().unwrap_or(String::from("null")),
            FieldType::Bool(b) => b.to_string(),
            FieldType::F64Option(f) => f.clone().unwrap_or(0.0).to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoPosition {
    pub latitude: String,
    pub longitude: String,
}

pub trait RandomGen {
    fn random<T: RngCore + ?Sized>(rng: &mut T) -> Self;
}

impl RandomGen for FakeData {
    fn random<T: RngCore + ?Sized>(rng: &mut T) -> Self {
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
    fn random<T: RngCore + ?Sized>(rng: &mut T) -> Self {
        GeoPosition {
            latitude: format!{"{:.7}", rng.gen_range(-90.0..90.0)},
            longitude: format!{"{:.7}", rng.gen_range(-180.0..180.0)}
        }
    }
}

impl FakeData {
    pub fn get_filtered_vec(&self, fields: &Vec<&str>) -> Vec<String> {
        return fields.iter().map(|field| match *field {
                "_type" => self._type.clone(),
                "_id" => self._id.to_string(),
                "key" => self.key.clone().unwrap_or(String::from("null")),
                "name" => self.name.clone(),
                "fullName" => self.full_name.clone(),
                "iata_airport_code" => self.iata_airport_code.clone().unwrap_or(String::from("null")),
                "type" => self.r#type.clone(),
                "country" => self.country.clone(),
                "latitude" => self.geo_position.latitude.clone(),
                "longitude" => self.geo_position.longitude.clone(),
                "location_id" => self.location_id.to_string(),
                "inEurope" => self.in_europe.to_string(),
                "countryCode" => self.country_code.clone(),
                "coreCountry" => self.core_country.to_string(),
                "distance" => self.distance.as_ref().unwrap_or(&0.0).to_string(),
                _ => String::from("None")
            }
        ).collect();
    }

    pub fn get_filtered_indexmap(&self, fields: &Vec<&str>) -> IndexMap<String, FieldType> {
        let mut map = IndexMap::new();
        for field in fields {match *field {
                "_type" => map.insert(String::from("_type"), FieldType::String(self._type.clone())),
                "_id" => map.insert(String::from("_id"), FieldType::U32(self._id)),
                "key" => map.insert(String::from("key"), FieldType::StringOption(self.key.clone())),
                "name" => map.insert(String::from("name"), FieldType::String(self.name.clone())),
                "fullName" => map.insert(String::from("fullName"), FieldType::String(self.full_name.clone())),
                "iata_airport_code" => map.insert(String::from("iata_airport_code"), FieldType::StringOption(self.iata_airport_code.clone())),
                "type" => map.insert(String::from("type"), FieldType::String(self.r#type.clone())),
                "country" => map.insert(String::from("country"), FieldType::String(self.country.clone())),
                "latitude" => map.insert(String::from("latitude"), FieldType::String(self.geo_position.latitude.clone())),
                "longitude" => map.insert(String::from("longitude"), FieldType::String(self.geo_position.longitude.clone())),
                "location_id" => map.insert(String::from("location_id"), FieldType::U32(self.location_id)),
                "inEurope" => map.insert(String::from("inEurope"), FieldType::Bool(self.in_europe)),
                "countryCode" => map.insert(String::from("countryCode"), FieldType::String(self.country_code.clone())),
                "coreCountry" => map.insert(String::from("coreCountry"), FieldType::Bool(self.core_country)),
                "distance" => map.insert(String::from("distance"), FieldType::F64Option(self.distance.clone())),
                _ => map.insert(String::from("None"), FieldType::String(String::from("None")))
            };
        }
        return map;
    }

    pub fn to_computed_vec(&self, fields: &Vec<&str>) -> Vec<String> {
        let used_fields: Vec<&str> = FIELDS.clone().into_iter().filter(|x| fields.iter().any(|y| {
            let re = Regex::new(&format!(r"\b{}\b", x)).unwrap();
            re.is_match(y)
        })).collect();
        let map = self.get_filtered_indexmap(&used_fields);
        Vec::new()
    }
}