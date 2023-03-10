use crate::models::prelude::User;
use crate::models::user;
use crate::routes::crud;
use actix_web::{web, Scope};
use crate::models::user::{ActiveModel};



pub struct UserRoutes {}

impl crud::CrudRoutes<User, ActiveModel, user::ModelWithoutId> for UserRoutes {
    fn entity() -> User {
        User
    }

    fn model() -> user::ActiveModel {
        user::ActiveModel::default()
    }

    fn export_routes() -> Scope {

        web::scope("/users")
            .route("/", web::get().to(Self::list))
            .route("/", web::post().to(Self::create))
            .route("/{id}/", web::delete().to(Self::delete))
            .route("/{id}/", web::get().to(Self::get))
            .route("/{id}/", web::patch().to(Self::update))
    }
}