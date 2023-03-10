extern crate log;

mod services;
mod routes;
mod models;

use crate::routes::crud::{CrudRoutes, DefaultRoutes};
use crate::routes::auth::AuthRoutes;

use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use services::db;
use services::google_auth::google_auth;
use services::jwt;
use routes::users::UserRoutes;
use env_logger;
use oauth2::basic::BasicClient;
use sea_orm::{DatabaseConnection, DbErr};
use actix_web_httpauth::middleware::HttpAuthentication;


pub struct AppState {
    db: DatabaseConnection,
    oauth: BasicClient
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        let new = self.to_owned();
        Self {db: new.db, oauth: new.oauth}
    }
}

async fn create_state() -> Result<AppState, DbErr> {
    match db::db_conn().await {
        Ok(conn) => {
            let oauth = google_auth::get_client();
            Ok(AppState {db: conn, oauth})
        }
        Err(err) => {
            Err(err)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    println!("Server starting in port 3000");
    HttpServer::new( move || {
        let auth = HttpAuthentication::bearer(jwt::bearer_auth_validator);
        App::new()
            .data_factory(|| { create_state() })
            .service(AuthRoutes::export_routes())
            .wrap(Logger::default())
            .service( UserRoutes::export_routes().wrap(auth))

    })
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}