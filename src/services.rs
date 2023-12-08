use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};

use actix_web::{get, HttpResponse, Responder, web::{Data, Query}};

use rand::prelude::*;
use rayon::prelude::*;
use serde::Deserialize;
use crate::{data_gen::{FakeData, RandomGen}, AppConfig, measure};
use csv::Writer;

#[derive(Deserialize)]
struct CSVFields {
    length: Option<usize>,
    fields: Option<String>,
    perf: Option<bool>,
}

#[derive(Deserialize)]
struct JSONFields {
    length: Option<usize>,
    perf: Option<bool>,
}


/// API endpoint to generate fake data in JSON format with arguments specified in `JSONFields` struct.
/// 
/// # Returns
/// 
/// Response with JSON data.
#[get("generate/json")]
pub async fn generate_data(args: Query<JSONFields>) -> impl Responder {
    fn generate_data_inner(size: usize) -> String{
        let data: Vec<FakeData> = (0..size)
            .into_par_iter()
            .map(|_| FakeData::random(&mut thread_rng()))
            .collect();
        serde_json::to_string(&data).unwrap()
    }
    let args = args.into_inner();
    let size = args.length.unwrap_or(10);
    let perf = args.perf.unwrap_or(false);

    let data = if perf {
        serde_json::to_string(measure!(generate_data_inner(size))).unwrap()
    } else {
        generate_data_inner(size)
    };

    HttpResponse::Ok()
    .content_type("application/json; charset=utf-8")
    .body(data)
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
        let result = match row.to_computed_vec(&fields) {
            Ok(data) => data,
            Err(err) => return HttpResponse::BadRequest().body(err),
        };
        writer.write_record(result).unwrap();
    };

    let csv = String::from_utf8(writer.into_inner().unwrap()).unwrap();

    HttpResponse::Ok()
    .content_type("text/csv; charset=utf-8")
    .body(csv)
}

// /// API endpoint to measure performance of handling CSV data generation with arguments specified in `CSVFields` struct.
// /// 
// /// # Returns
// /// 
// /// Response with performance data.
// pub async fn measure_csv_perf(data: Data<AppConfig>, info: Query<CSVFields>) -> impl Responder {
//     let args = info.into_inner();
//     let size = args.length.unwrap_or(10);

    
// }