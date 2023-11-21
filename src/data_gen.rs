use rand::prelude::*;
use serde::Serialize;


#[derive(Serialize)]
pub struct FakeData {
    _type: String,
    _id: u64,
    key: Option<String>,
    name: String,
    #[serde(rename = "fullName")]
    full_name: String,
    iata_airport_code: Option<String>,
    r#type: String,
    country: String,
    geo_position: GeoPosition,
    location_id: u64,
    #[serde(rename = "inEurope")]
    in_europe: bool,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[serde(rename = "coreCountry")]
    core_country: bool,
    distance: Option<f64>
}

#[derive(Serialize)]
pub struct GeoPosition {
    latitude: f64,
    longitude: f64,
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
            _id: rng.next_u64(),
            key: Option::None,
            name: String::from(random_street_name),
            full_name: String::from(random_street_name)+", "+&random_country.0,
            iata_airport_code: Option::None,
            r#type: String::from("location"),
            country: String::from(random_country.0),
            geo_position: GeoPosition::random(rng),
            location_id: rng.next_u64(),
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
            latitude: rng.gen_range(-90.0..90.0),
            longitude: rng.gen_range(-180.0..180.0)
        }
    }
}