use std::time::Duration;
use sea_orm::{Database, ConnectOptions, DatabaseConnection, DbErr};

pub async fn db_conn() -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new("sqlite://data.sqlite?mode=rwc".to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    Database::connect(opt).await

}
