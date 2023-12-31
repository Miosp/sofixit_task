use actix_web::{HttpServer, App, web::Data};


mod data_gen;
mod services;
mod expression_parser;
mod performance_measure;

#[derive(Clone)]
struct AppConfig {
    root: String,
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig {
        root: String::from("127.0.0.1"),
        port: 8080,
    };
    let server_config = config.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(server_config.clone()))
            .service(services::generate_data)
            .service(services::data_to_csv)
            .service(services::measure_csv_perf)
    }).bind((config.root.clone(), config.port.clone()))?
    .run()
    .await
}
