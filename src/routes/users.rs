use std::future::Future;
use crate::models::prelude::User;
use actix_web::{web, Scope};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Select};
use sea_orm::sea_query::ColumnRef::Column;
use crate::models::user;
use crate::models::user::{Entity, Model};
use crate::routes::crud;

pub struct UserRoutes {}

impl crud::CrudRoutes<User, user::ActiveModel, user::ModelWithoutId> for UserRoutes {
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