use std::fmt;
use oauth2::{AuthorizationCode};
use crate::services::google_auth::google_auth;
use serde::{Deserialize, Serialize};
use crate::AppState;
use actix_web::{HttpResponse, Scope, web};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, IntoActiveModel};
use sea_orm::ActiveValue::Set;
use crate::models::prelude::User;
use crate::models::user;
use crate::models::user::ActiveModel;
use crate::routes::crud::DefaultRoutes;
use crate::services::jwt::{Claims};


#[derive(Deserialize, Serialize)]
pub struct OAuthCallbackParams {
    code: String,
    state: String,
    scope: String
}

impl fmt::Display for OAuthCallbackParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Code: {}, state: {}", self.code, self.state)
    }
}

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    token: String,
}


pub struct AuthRoutes {}

impl AuthRoutes {
    async fn find_or_register(user: Claims, db: &DatabaseConnection) -> Result<ActiveModel, DbErr> {
        match User::find().filter(user::Column::Username.contains(user.sub.as_str())).one(&db.clone()).await {
            Ok(result) => {
                match result {
                    None => {
                        let model = ActiveModel {
                            id: Default::default(),
                            username: Set(user.sub),
                            service: Set("google".to_string()),
                        };
                        match model.save(db).await {
                            Ok(saved) => {
                                Ok(saved)
                            }
                            Err(err) => Err(err)
                        }
                    }
                    Some(model) => Ok(model.into_active_model())
                }
            }
            Err(err) => Err(err)
        }
    }

    async fn login(data: web::Data<AppState>) -> HttpResponse {
        let auth_url = google_auth::get_auth_url(&data.oauth);
        HttpResponse::Found().append_header(("Location", auth_url)).finish()
    }


    pub async fn callback(query_params: web::Query<OAuthCallbackParams>, data: web::Data<AppState>) -> HttpResponse {
        let code = AuthorizationCode::new(query_params.code.clone());
        let token = google_auth::get_token(&data.oauth, code).await;
        match token {
            Ok(t) => {
                let info = google_auth::get_user_info(t).await;
                match info {
                    Ok(profile) => {
                        println!("Profile: {}", profile.id);
                        let claim = Claims {
                            id: profile.id.to_string(),
                            sub: profile.email.to_string(),
                            app: "actix".to_string(),
                            exp: 10000000000,
                        };
                        let token = claim.encode();
                        match Self::find_or_register(claim, &data.db).await {
                            Ok(_) => HttpResponse::Ok().json(TokenResponse {token: token.unwrap().to_string()}),
                            Err(err) => HttpResponse::InternalServerError().json(err.to_string())
                        }

                    },
                    Err(err) => HttpResponse::InternalServerError().json(err.to_string())
                }
            }
            Err(err) => HttpResponse::InternalServerError().json(err.to_string())
        }
    }
}

impl DefaultRoutes for AuthRoutes {
    fn export_routes() -> Scope {
        web::scope("/auth")
            .route("/login", web::get().to(Self::login))
            .route("/google/callback", web::get().to(Self::callback))
    }

}



