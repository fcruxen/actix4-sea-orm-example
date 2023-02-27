use actix_web::{HttpResponse, Scope, web};
use actix_web::http::StatusCode;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, IntoActiveModel, ModelTrait, PrimaryKeyTrait, Value};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::ColumnRef::Column;
use sea_orm::sea_query::ValueTuple;
use serde_json::json;
use json_value_merge::Merge;
use serde_json::Value as JsonValue;

#[async_trait]
pub trait CrudRoutes<T, M, N>
    where
        T: EntityTrait,
        T::Model: Serialize,
        N: IntoActiveModel<M> + Serialize + Sync + Send + 'static ,
        M: ActiveModelTrait + ActiveModelBehavior + Send + Sync + From<<T as EntityTrait>::Model>,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<M>,
        <<M as ActiveModelTrait>::Entity as EntityTrait>::Model: for<'de> Deserialize<'de>,
        <<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<i32>,
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

    async fn update(
        path: web::Path<(i32,)>,
        record: web::Json<N>,
        data: web::Data<DatabaseConnection>,
    ) -> HttpResponse {
        let id: i32 = path.into_inner().0;
        let mut existing_record: M = match T::find_by_id(id).one(data.as_ref()).await {
            Ok(Some(record)) => record.into(),
            Ok(None) => {
                return Self::default_err(DbErr::Custom(format!("Could not find id: {}", id)));
            }
            Err(err) => {
                return Self::default_err(err);
            }
        };
        log::debug!("Record found, changing it");
        let mut new_record = record.into_inner().into_active_model();
        let mut merged_data = JsonValue::new_object();
        merged_data.merge(existing_record.to_serialize()).unwrap();
        merged_data.merge(new_record.to_serialize()).unwrap();
        new_record = M::from(merged_data).unwrap();
        let j = json!({"id": id});
        match  update_record {
            Ok(..) => {
                match new_record.update(data.as_ref()).await {
                    Ok(_) => HttpResponse::Ok().status(StatusCode::ACCEPTED).body(""),
                    Err(err) => Self::default_err(err),
                }
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