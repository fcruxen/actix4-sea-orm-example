use std::fmt::Debug;
use std::str::FromStr;
use actix_web::{HttpResponse, Scope, web};
use actix_web::http::StatusCode;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ColumnTypeTrait, DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, IntoActiveModel, ModelTrait, PrimaryKeyTrait, Value};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::ColumnRef::Column;
use sea_orm::sea_query::ValueTuple;
use serde_json::json;
use json_value_merge::Merge;
use sea_orm::sea_query::extension::postgres::IntoTypeRef;
use serde_json::Value as JsonValue;


#[async_trait]
pub trait CrudRoutes<T, M, N>
    where
        T: EntityTrait,
        T::Model: Serialize,
        N: IntoActiveModel<M> + Serialize + Sync + Send + 'static + Clone,
        M: ActiveModelTrait + ActiveModelBehavior + Send + Sync + From<<T as EntityTrait>::Model>,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<M>,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: for<'de> Deserialize<'de>,
        <<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<i32>,
        <T as EntityTrait>::Model: Sync,
        <<<M as ActiveModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug
{
    fn entity() -> T;
    fn model() -> M;
    fn primary_column() -> <<M as ActiveModelTrait>::Entity as EntityTrait>::Column {
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Column::from_str("id").unwrap()
    }
    fn export_routes() -> Scope;

    async fn update(
        path: web::Path<(i32,)>,
        record: web::Json<N>,
        data: web::Data<DatabaseConnection>,
    ) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        match T::find_by_id(id).one(data.as_ref()).await {
            Ok(_) => {
                let mut updated_record = record.into_inner().clone().into_active_model();
                updated_record.set(Self::primary_column(), Value::Int(Some(id)));
                match updated_record.save(data.as_ref()).await {
                    Ok(_) => HttpResponse::Ok().status(StatusCode::ACCEPTED).json(""),
                    Err(err) => Self::default_err(err)
                }
            },
            Err(err) => {
                return Self::default_err(err);
            }
        }
    }

    fn default_err(err: DbErr) -> HttpResponse {
        HttpResponse::BadRequest()
            .content_type("application/json")
            .json(serde_json::json!({"error": err.to_string()}))
    }

    async fn list(data: web::Data<DatabaseConnection>) -> HttpResponse {
        match Self::entity().select().all(data.as_ref()).await {
            Ok(list) => {
                HttpResponse::Ok().json(list)
            }
            Err(err) => Self::default_err(err)
        }
    }

    async fn get(path: web::Path<(i32,)>, data: web::Data<DatabaseConnection>) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        match  T::find_by_id(id).one(data.as_ref()).await {
            Ok(record) => {
                HttpResponse::Ok().json(record)
            }
            Err(err) => Self::default_err(err)
        }
    }




    async fn create(record: web::Json<N>, data: web::Data<DatabaseConnection>) -> HttpResponse {
        let new_record = record.into_inner().into_active_model();
        match new_record.save(data.as_ref()).await {
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

    async fn delete(path: web::Path<(i32,)>, data: web::Data<DatabaseConnection>) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        match T::find_by_id(id).one(data.as_ref()).await {
            Ok(r) => {
                match r {
                    None => Self::default_err(DbErr::Custom(format!("Cound not find id: {}", id).to_string())),
                    Some(v) => {
                        let m: M  = v.into();
                        let op = m.delete(data.as_ref()).await;
                        match op {
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