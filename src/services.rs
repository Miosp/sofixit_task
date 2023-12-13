use std::time::Instant;

use actix_web::{get, HttpResponse, Responder, web::{Data, Query, Path}};

use rand::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{data_gen::{FakeData, RandomGen, FIELDS}, AppConfig, measure, measure_async, expression_parser};
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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct JSONResponsePerf {
    data: Vec<FakeData>,
    #[serde(rename = "JSONcpuUtil")]
    json_cpu_util: Vec<f32>,
    #[serde(rename = "JSONmemUtil")]
    json_mem_util: Vec<u64>
}

impl From<(Vec<FakeData>, Vec<f32>, Vec<u64>)> for JSONResponsePerf {
    fn from(data: (Vec<FakeData>, Vec<f32>, Vec<u64>)) -> Self {
        JSONResponsePerf {
            data: data.0,
            json_cpu_util: data.1,
            json_mem_util: data.2,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CSVResponsePerf {
    csv: String,
    #[serde(rename = "CSVcpuUtil")]
    csv_cpu_util: Vec<f32>,
    #[serde(rename = "CSVmemUtil")]
    csv_mem_util: Vec<u64>,
    #[serde(rename = "JSONcpuUtil")]
    json_cpu_util: Vec<f32>,
    #[serde(rename = "JSONmemUtil")]
    json_mem_util: Vec<u64>,
    #[serde(rename = "JSONtime")]
    json_time: u128,
}

/// API endpoint to generate fake data in JSON format with arguments specified in `JSONFields` struct.
/// 
/// # Returns
/// 
/// Response with JSON data.
#[get("generate/json/{length}")]
pub async fn generate_data(path: Path<u32>, args: Query<JSONFields>) -> impl Responder {
    fn generate_data_inner(size: usize) -> Vec<FakeData>{
        (0..size)
            .into_par_iter()
            // .into_iter()
            .map(|_| FakeData::random(&mut thread_rng()))
            .collect()
    }
    let args = args.into_inner();
    let size = path.into_inner() as usize;
    let perf = args.perf.unwrap_or(false);

    let data = if perf {
        let result = JSONResponsePerf::from(measure!(generate_data_inner(size)));
        serde_json::to_string(&result).unwrap()
    } else {
        serde_json::to_string(&generate_data_inner(size)).unwrap()
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
    async fn data_to_csv_inner(perf: bool, size: usize, fields: Vec<String>, data: Data<AppConfig>) -> Result<(String, (Vec<f32>, Vec<u64>), u128), String> {
        let req_path = if perf {
            format!("http://{}:{}/generate/json/{}?perf=true", data.root, data.port, size)
        } else {
            format!("http://{}:{}/generate/json/{}", data.root, data.port, size)
        };
    
        let timer = Instant::now();
        let resp = reqwest::get(req_path).await;
        let elapsed = timer.elapsed().as_millis();
        if resp.is_err() {
            return Err(format!("Failed to get data from server: {}", resp.unwrap_err()));
        }

        let resp = if perf {
            match resp.unwrap().json::<JSONResponsePerf>().await {
                Ok(data) => data,
                Err(_) => return Err(String::from("Failed to parse JSON response")),
            }
        } else {
            match resp.unwrap().json::<Vec<FakeData>>().await {
                Ok(data) => JSONResponsePerf::from((data, vec![], vec![])),
                Err(_) => return Err(String::from("Failed to parse JSON response")),
            }
        };

        //precompute parsed fields
        let used_fields: Vec<&str> = FIELDS.clone().into_iter().filter(|x| fields.iter().any(|y| {
            let re = regex::Regex::new(&format!(r"\b{}\b", x)).unwrap();
            re.is_match(y)
        })).collect();
        let parsed_fields = fields.iter().map(|field| expression_parser::parse_expression(field))
            .collect::<Result<Vec<expression_parser::Expression>, String>>()?;

        let mut writer = Writer::from_writer(vec![]);
        writer.write_record(&fields).unwrap();
        for row in resp.data {
            let map = row.get_filtered_indexmap(&used_fields);
            let result: Vec<String> = parsed_fields.iter()
                .map(|field| {
                    field.eval(&map).unwrap().to_string()
                }).collect();
            writer.write_record(result).unwrap();
        };
        let csv = String::from_utf8(writer.into_inner().unwrap()).unwrap();
    
        if perf {
            Ok((csv, (resp.json_cpu_util, resp.json_mem_util), elapsed))
        } else {
            Ok((csv, (vec![], vec![]), 0))
        }
    }
    let args = info.into_inner();
    let size = path.into_inner() as usize;
    let fields = args.fields.unwrap_or(String::from("type, _id, name, latitude, longitude"));
    let fields: Vec<String> = fields.split(',').map(|x| x.trim().to_string()).collect();
    let perf = args.perf.unwrap_or(false);

    if perf {
        let res = measure_async!(data_to_csv_inner(perf, size, fields, data));
        if res.0.is_err() { return HttpResponse::InternalServerError().body(res.0.unwrap_err()); }
        let jsonres = res.0.unwrap();

        HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json(CSVResponsePerf {
            csv: jsonres.0,
            csv_cpu_util: res.1,
            csv_mem_util: res.2,
            json_cpu_util: jsonres.1.0,
            json_mem_util: jsonres.1.1,
            json_time: jsonres.2,
        })
    } else {
        let res = data_to_csv_inner(perf, size, fields, data).await;
        if res.is_err() { return HttpResponse::InternalServerError().body(res.unwrap_err()); }

        HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .body(res.unwrap().0)
    }
}

/// API endpoint to measure performance of handling CSV data generation with arguments specified in `CSVFields` struct.
/// 
/// # Returns
/// 
/// Response with performance data.
#[get("measure/csv/{length}")]
pub async fn measure_csv_perf(path: Path<u32>, data: Data<AppConfig>, info: Query<CSVFields>) -> impl Responder {
    let args = info.into_inner();
    let length = path.into_inner() as usize;
    let fields = args.fields;

    let req_path = match fields {
        Some(fields) => format!("http://{}:{}/generate/csv/{}?perf=true&fields={}", data.root, data.port, length, fields),
        None => format!("http://{}:{}/generate/csv/{}?perf=true", data.root, data.port, length),
    };

    let timer = Instant::now();
    let resp = reqwest::get(req_path).await;
    let elapsed = timer.elapsed().as_millis();
    if resp.is_err() {
        return HttpResponse::InternalServerError().body(format!("Failed to get data from server: {}", resp.unwrap_err()));
    }
    let result = resp.unwrap().json::<CSVResponsePerf>().await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("Failed to parse JSON response: {}", result.unwrap_err()));
    }
    let result = result.unwrap();

    let response = format!(r#"STATISTICS FOR CALLING /generate/csv/{length}:
- CPU utilization: {:?}
- Memory utilization: {:?}
- Time elapsed: {} ms

STATISTICS FOR CALLING /generate/json/{length} from /generate/csv/{length}:
- CPU utilization: {:?}
- Memory utilization: {:?}
- Time elapsed: {} ms

All utilization values are measured with 200ms interval.
"#, result.csv_cpu_util, result.csv_mem_util, elapsed, result.json_cpu_util, result.json_mem_util, result.json_time);

    HttpResponse::Ok()
    .content_type("text/plain; charset=utf-8")
    .body(response)
}