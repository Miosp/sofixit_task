use actix_web::{get, HttpResponse, Responder, web::{Path, Data}};

use rand::prelude::*;
use rayon::prelude::*;
use crate::{data_gen::{FakeData, RandomGen}, AppConfig, data_csv::CSVData};
use csv::Writer;

#[get("generate/json/{size}")]
pub async fn generate_data(path: Path<usize>) -> impl Responder {
    let size: usize = path.into_inner();
    let data = (0..size).into_par_iter().map(|_| FakeData::random(&mut thread_rng())).collect::<Vec<FakeData>>();
    HttpResponse::Ok().json(data)
}

#[get("generate/csv/{size}")]
pub async fn data_to_csv(path: Path<usize>, data: Data<AppConfig>) -> impl Responder {
    let req_path = format!("http://{}:{}/generate/json/{}", data.root, data.port, path.into_inner());
    let resp = reqwest::get(req_path).await;
    if resp.is_err() {
        return HttpResponse::BadRequest().body(format!("{:?}", resp.unwrap_err()));
    }
    let resp = match resp.unwrap().json::<Vec<FakeData>>().await {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().body("Failed to parse JSON response"),
    
    };

    let mut writer = Writer::from_writer(vec![]);
    for row in resp {
        if let Err(e) = writer.serialize(CSVData::from(row)) {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    }

    let csv = String::from_utf8(writer.into_inner().unwrap()).unwrap();

    HttpResponse::Ok().body(csv)
}