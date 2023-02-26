use std::fmt::{Debug, Display};
use actix_web::{HttpResponse, Scope, web};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sea_orm::sea_query::ColumnSpec::Default;
use sea_orm::sea_query::ValueTuple;
use serde_json::json;


#[async_trait]
pub trait CrudRoutes<T, M>
    where
        T: EntityTrait,
        T::Model: Serialize,
        M: ActiveModelTrait + ActiveModelBehavior + Send,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<M>,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: for<'de> Deserialize<'de>,
        <T as EntityTrait>::Model: Sync
{
    fn entity() -> T;
    fn model() -> M;
    fn export_routes() -> Scope;

    fn default_err(err: DbErr) -> HttpResponse {
        HttpResponse::BadRequest()
            .content_type("application/json")
            .json(serde_json::json!({"error": err.to_string()}))
    }

    async fn list(data: web::Data<DatabaseConnection>) -> HttpResponse {
        match Self::entity().select().all(data.as_ref()).await {
            Ok(list) => {
                log::debug!("Returning JSON");
                HttpResponse::Ok().json(list)
            }
            Err(err) => Self::default_err(err)
        }
    }

    async fn create(record: web::Json<T::Model>, data: web::Data<DatabaseConnection>) -> HttpResponse {
        let mut new_record = Self::model();
        match new_record.set_from_json(json!(record)) {
            Ok(_) => {
                match new_record.save(data.as_ref()).await {
                    Ok(r) => {
                        HttpResponse::Ok().json("")
                    },
                    Err(err) => Self::default_err(err)
                }
            }
            Err(err) => Self::default_err(err)
        }
    }
}