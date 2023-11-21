use actix_web::{HttpServer, App};


mod data_gen;
use crate::data_gen::generate_data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(generate_data)
    }).bind(("127.0.0.1", 8080))?
    .run()
    .await
}
