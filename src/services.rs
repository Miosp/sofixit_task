use actix_web::{get, HttpResponse, Responder, web::{Data, Query}};

use rand::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use crate::{data_gen::{FakeData, RandomGen, FIELDS}, AppConfig};
use csv::Writer;

#[derive(Deserialize)]
struct CSVFields {
    length: Option<usize>,
    fields: Option<String>,
}

#[derive(Deserialize)]
struct JSONFields {
    length: Option<usize>,
}


/// API endpoint to generate fake data in JSON format with arguments specified in `JSONFields` struct.
/// 
/// # Returns
/// 
/// Response with JSON data.
#[get("generate/json")]
pub async fn generate_data(args: Query<JSONFields>) -> impl Responder {
    let size = args.into_inner().length.unwrap_or(10);
    let data: Vec<FakeData> = (0..size).into_par_iter().map(|_| FakeData::random(&mut thread_rng())).collect();
    HttpResponse::Ok()
    .content_type("application/json; charset=utf-8")
    .json(data)
}


/// API endpoint to convert JSON data to CSV format with arguments specified in `CSVFields` struct.
/// 
/// # Returns
/// 
/// Response with CSV data.
#[get("generate/csv")]
pub async fn data_to_csv(data: Data<AppConfig>, info: Query<CSVFields>) -> impl Responder {
    let args = info.into_inner();
    let size = args.length.unwrap_or(10);
    let fields_string = args.fields.unwrap_or(String::from("type, _id, name, latitude, longitude"));
    let fields: Vec<&str> = fields_string.split(',').map(|x| x.trim()).collect();

    // Make request to generator service
    let req_path = format!("http://{}:{}/generate/json?length={}", data.root, data.port, size);
    let resp = reqwest::get(req_path).await;
    if resp.is_err() {
        return HttpResponse::BadRequest().body(format!("{:?}", resp.unwrap_err()));
    }
    let resp = match resp.unwrap().json::<Vec<FakeData>>().await {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().body("Failed to parse JSON response"),
    };

    // Write CSV data
    let mut writer = Writer::from_writer(vec![]);
    writer.write_record(&fields).unwrap();
    for row in resp {
        writer.write_record(row.get_filtered_vec(&fields)).unwrap();
    }

    let csv = String::from_utf8(writer.into_inner().unwrap()).unwrap();

    HttpResponse::Ok()
    .content_type("text/csv; charset=utf-8")
    .body(csv)
}