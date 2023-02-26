#[macro_use]
extern crate log;

mod services;
mod routes;
mod models;


use std::process::exit;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web::web::Data;
use services::db;
use routes::users::UserRoutes;
use routes::crud;
use env_logger;
use crate::routes::crud::CrudRoutes;


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let db_conn = db::db_conn().await;
    match db_conn {
        Ok(db) => {
            println!("Server starting in port 8080");
            HttpServer::new( move || {
                App::new()
                    .app_data(Data::new(db.clone()))
                    .service(index)
                    .service( UserRoutes::export_routes())

            })
                .bind(("0.0.0.0", 8080))?
                .run()
                .await
        }
        Err(err) => {
            println!("DB ERROR: {}", err.to_string());
            exit(1);
        }
    }

}