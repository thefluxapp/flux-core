use std::sync::Arc;

use anyhow::Error;
use async_nats::jetstream;
use sea_orm::{ConnectOptions, Database, DbConn};

use super::{settings::AppSettings, AppJS};

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub db: Arc<DbConn>,
    pub js: Arc<AppJS>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let nats = async_nats::connect(&settings.nats.endpoint).await?;
        let js = Arc::new(jetstream::new(nats));

        let mut opt = ConnectOptions::new(&settings.db.endpoint);
        opt.sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Debug);
        let db = Arc::new(Database::connect(opt).await?);

        Ok(Self { settings, js, db })
    }
}
