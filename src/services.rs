use std::time::Instant;

use actix_web::{get, HttpResponse, Responder, web::{Data, Query, Path}};

use rand::prelude::*;
use rayon::prelude::*;
use serde::Deserialize;
use crate::{data_gen::{FakeData, RandomGen}, AppConfig, measure};
use csv::Writer;

#[derive(Deserialize)]
struct CSVFields {
    fields: Option<String>,
    perf: Option<bool>,
}

#[derive(Deserialize)]
struct JSONFields {
    perf: Option<bool>,
}

/// API endpoint to generate fake data in JSON format with arguments specified in `JSONFields` struct.
/// 
/// # Returns
/// 
/// Response with JSON data.
#[get("generate/json/{length}")]
pub async fn generate_data(path: Path<u32>, args: Query<JSONFields>) -> impl Responder {
    fn generate_data_inner(size: usize) -> String{
        let data: Vec<FakeData> = (0..size)
            .into_par_iter()
            .map(|_| FakeData::random(&mut thread_rng()))
            .collect();
        serde_json::to_string(&data).unwrap()
    }
    let args = args.into_inner();
    let size = path.into_inner() as usize;
    let perf = args.perf.unwrap_or(false);

    let data = if perf {
        let result = measure!(generate_data_inner(size));
        serde_json::to_string(&result).unwrap()
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
#[get("generate/csv/{length}")]
pub async fn data_to_csv(path: Path<u32>, data: Data<AppConfig>, info: Query<CSVFields>) -> impl Responder {
    fn data_to_csv_inner(perf: bool, size: usize, fields: Vec<String>, data: Data<AppConfig>) -> Result<String, String> {
        let req_path = if perf {
            format!("http://{}:{}/generate/json/{}?perf=true", data.root, data.port, size)
        } else {
            format!("http://{}:{}/generate/json/{}", data.root, data.port, size)
        };
    
        let timer = Instant::now();
        let resp = reqwest::blocking::get(req_path);
        let elapsed = timer.elapsed().as_millis();
        if resp.is_err() {
            return Err(format!("Failed to get data from server: {}", resp.unwrap_err()));
        }
        let resp = if perf {
            match resp.unwrap().json::<(Vec<FakeData>, f32, u64)>() {
                Ok(data) => data,
                Err(_) => return Err(String::from("Failed to parse JSON response")),
            }
        } else {
            match resp.unwrap().json::<Vec<FakeData>>() {
                Ok(data) => (data, 0.0, 0),
                Err(_) => return Err(String::from("Failed to parse JSON response")),
            }
        };
        let mut writer = Writer::from_writer(vec![]);
        writer.write_record(&fields).unwrap();
        for row in resp.0 {
            let result = match row.to_computed_vec(&fields) {
                Ok(data) => data,
                Err(err) => return Err(format!("Failed to convert data to CSV: {}", err)),
            };
            writer.write_record(result).unwrap();
        };
        let csv = String::from_utf8(writer.into_inner().unwrap()).unwrap();
    
        if perf {
            Ok(serde_json::to_string(&(csv, (resp.1, resp.2), elapsed)).unwrap())
        } else {
            Ok(csv)
        }
    }
    let args = info.into_inner();
    let size = path.into_inner() as usize;
    let fields = args.fields.unwrap_or(String::from("type, _id, name, latitude, longitude"));
    let fields: Vec<String> = fields.split(',').map(|x| x.trim().to_string()).collect();
    let perf = args.perf.unwrap_or(false);

    if perf {
        let res = measure!(data_to_csv_inner(perf, size, fields, data));
        if res.0.is_err() { return HttpResponse::InternalServerError().body(res.0.unwrap_err()); }

        HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json((res.0.unwrap(), res.1, res.2))
    } else {
        let res = data_to_csv_inner(perf, size, fields, data);
        if res.is_err() { return HttpResponse::InternalServerError().body(res.unwrap_err()); }

        HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .body(res.unwrap())
    }
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