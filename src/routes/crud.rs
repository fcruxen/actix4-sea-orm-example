use std::fmt::Debug;
use std::str::FromStr;
use actix_web::{HttpResponse, Scope, web};
use actix_web::http::StatusCode;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DbErr, EntityOrSelect, EntityTrait, IntoActiveModel, PrimaryKeyTrait, Value};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sea_orm::sea_query::ValueTuple;
use crate::AppState;


pub trait DefaultRoutes {
    fn export_routes() -> Scope;
}

#[async_trait]
pub trait CrudRoutes<T, M, N>
    where
        T: EntityTrait,
        T::Model: Serialize,
        N: IntoActiveModel<M> + Serialize + Sync + Send + 'static + Clone,
        M: ActiveModelTrait + ActiveModelBehavior + Send + Sync + From<<T as EntityTrait>::Model>,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<M> + for<'de> Deserialize<'de>,
        <<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<i32>,
        <<<M as ActiveModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug
{
    fn entity() -> T;
    fn model() -> M;
    fn export_routes() -> Scope;

    fn primary_column() -> <<M as ActiveModelTrait>::Entity as EntityTrait>::Column {
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Column::from_str("id").unwrap()
    }

    fn default_err(err: DbErr) -> HttpResponse {
        HttpResponse::BadRequest()
            .content_type("application/json")
            .json(serde_json::json!({"error": err.to_string()}))
    }

    async fn update(path: web::Path<(i32,)>, record: web::Json<N>, data: web::Data<AppState>) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        match T::find_by_id(id).one(&data.db).await {
            Ok(_) => {
                let mut updated_record = record.into_inner().clone().into_active_model();
                updated_record.set(Self::primary_column(), Value::Int(Some(id)));
                match updated_record.save(&data.db).await {
                    Ok(_) => HttpResponse::Ok().status(StatusCode::ACCEPTED).json(""),
                    Err(err) => Self::default_err(err)
                }
            },
            Err(err) => {
                return Self::default_err(err);
            }
        }
    }

    async fn list(data: web::Data<AppState>) -> HttpResponse {
        match Self::entity().select().all(&data.db).await {
            Ok(list) => {
                HttpResponse::Ok().json(list)
            }
            Err(err) => Self::default_err(err)
        }
    }

    async fn get(path: web::Path<(i32,)>, data: web::Data<AppState>) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        match  T::find_by_id(id).one(&data.db).await {
            Ok(record) => {
                HttpResponse::Ok().json(record)
            }
            Err(err) => Self::default_err(err)
        }
    }

    async fn create(record: web::Json<N>, data: web::Data<AppState>) -> HttpResponse {
        let new_record = record.into_inner().into_active_model();
        match new_record.save(&data.db).await {
            Ok(r) => {
                match r.get_primary_key_value() {
                    None =>  HttpResponse::Ok().json(""),
                    Some(id) => {
                        match id {
                            ValueTuple::One(i) =>  HttpResponse::Ok().json(i.to_string()),
                            _ => HttpResponse::Ok().json("")
                        }
                    }
                }
            },
            Err(err) => Self::default_err(err)
        }
    }

    async fn delete(path: web::Path<(i32,)>, data: web::Data<AppState>) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        match T::find_by_id(id).one(&data.db).await {
            Ok(r) => {
                match r {
                    None => Self::default_err(DbErr::Custom(format!("Cound not find id: {}", id).to_string())),
                    Some(v) => {
                        let m: M  = v.into();
                        match m.delete(&data.db).await {
                            Ok(_) => HttpResponse::Ok().status(StatusCode::ACCEPTED).body(""),
                            Err(err) => Self::default_err(err)
                        }
                    }
                }
            }
            Err(err) => Self::default_err(err)
        }
    }
}