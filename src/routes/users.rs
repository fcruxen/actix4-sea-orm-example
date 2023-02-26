use crate::models::prelude::User;
use actix_web::{web, Scope};
use crate::models::user;
use crate::routes::crud;

pub struct UserRoutes {}

impl crud::CrudRoutes<User, user::ActiveModel> for UserRoutes {
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
    }
}

/*
async fn list(data: web::Data<DatabaseConnection>) -> HttpResponse {
    match User::find().all(data.as_ref()).await {
        Ok(list) => {
            HttpResponse::Ok().json(list)
        }
        Err(err) => {
            HttpResponse::BadRequest().body(err.to_string())
        }
    }
}

async fn create(record: web::Json<user::ModelWithoutId>, data: web::Data<DatabaseConnection>) -> HttpResponse {
    user::ActiveModel::from_json(json!(record)).expect("TODO: panic message");
    let user = user::ActiveModel {
        id: Default::default(),
        username: Set(record.username.to_owned()),
        service: Set(record.service.to_owned())
    };
    match user.save(data.as_ref()).await {
        Ok(result) => {
            HttpResponse::Ok().json("")
        }
        Err(err) => {
            log::debug!("ERROR CREATING ACTIVE MODEL");
            HttpResponse::BadRequest().body(err.to_string())
        }
    }

}
*/