use anyhow::Error;
use sea_orm::{DbConn, TransactionTrait};
use summarize_stream::SummarizeStreamRequest;

use super::repo;

pub async fn get_streams(db: &DbConn) -> Result<GetStreamsResponse, Error> {
    let streams = repo::find_streams(db).await?;

    Ok(GetStreamsResponse { streams })
}

pub async fn summarize_stream(db: &DbConn, request: SummarizeStreamRequest) -> Result<(), Error> {
    let txn = db.begin().await?;

    repo::update_stream_text(&txn, request.stream_id, request.text, request.version).await?;

    txn.commit().await?;

    Ok(())
}

pub struct GetStreamsResponse {
    pub streams: Vec<repo::stream::Model>,
}

pub mod summarize_stream {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Deserialize, Debug)]
    pub struct SummarizeStreamRequest {
        pub stream_id: Uuid,
        pub text: String,
        pub version: DateTime<Utc>,
    }
}
