use crate::app::{error::AppError, settings::AppSettings, AppJS};
use anyhow::Error;
use bytes::BytesMut;
use chrono::Utc;
use create_message::{Request, Response};
use flux_core_api::{summarize_stream_request::Message as StreamMessage, SummarizeStreamRequest};
use prost::Message;
use sea_orm::{DbConn, TransactionTrait as _};
use uuid::Uuid;

use super::repo;

pub async fn get_message(
    db: &DbConn,
    req: get_message::Request,
) -> Result<get_message::Response, Error> {
    let message = repo::find_message_by_id(db, req.message_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let stream = repo::find_stream_by_message_id(db, req.message_id).await?;
    let prev_stream = match stream {
        Some(ref stream) => repo::find_prev_stream_by_message_id(db, message.id, stream.id).await?,
        None => None,
    };

    let message = (message, prev_stream);

    let messages = match stream {
        Some(stream) => repo::find_messages_by_stream_id(db, stream.id).await?,
        None => vec![message.clone()],
    };

    Ok(get_message::Response { message, messages })
}

pub mod get_message {
    use uuid::Uuid;

    use crate::app::messages::repo;

    pub struct Request {
        pub message_id: Uuid,
    }
    pub struct Response {
        pub message: (repo::message::Model, Option<repo::stream::Model>),
        pub messages: Vec<(repo::message::Model, Option<repo::stream::Model>)>,
    }
}

pub async fn create_message(db: &DbConn, request: Request) -> Result<Response, Error> {
    let txn = db.begin().await?;

    let message = repo::create_message(
        &txn,
        repo::message::Model {
            id: Uuid::now_v7(),
            text: request.text.clone(),
            user_id: request.user_id,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        },
    )
    .await?;

    let stream = match request.message_id {
        Some(message_id) => {
            repo::find_message_by_id(&txn, message_id)
                .await?
                .ok_or(AppError::NotFound)?;

            let stream = repo::create_stream(
                &txn,
                repo::stream::Model {
                    id: Uuid::now_v7(),
                    title: None,
                    text: None,
                    message_id,
                    is_main: false,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                },
            )
            .await?
            .ok_or(AppError::NotFound)?;

            repo::create_message_stream(
                &txn,
                repo::message_stream::Model {
                    id: Uuid::now_v7(),
                    message_id,
                    stream_id: stream.id,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                },
            )
            .await?;

            repo::create_message_stream(
                &txn,
                repo::message_stream::Model {
                    id: Uuid::now_v7(),
                    message_id: message.id,
                    stream_id: stream.id,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                },
            )
            .await?;

            repo::create_stream_user(
                &txn,
                repo::stream_user::Model {
                    id: Uuid::now_v7(),
                    stream_id: stream.id,
                    user_id: request.user_id,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                },
            )
            .await?;

            Some(stream)
        }
        None => None,
    };

    txn.commit().await?;

    Ok(Response { message, stream })
}

pub mod create_message {
    use uuid::Uuid;
    use validator::Validate;

    use crate::app::messages::repo::{message, stream};

    #[derive(Validate)]
    pub struct Request {
        pub text: String,
        pub user_id: Uuid,
        pub message_id: Option<Uuid>,
    }
    pub struct Response {
        pub message: message::Model,
        pub stream: Option<stream::Model>,
    }
}

pub async fn summarize_stream_by_message_id(
    db: &DbConn,
    js: &AppJS,
    settings: AppSettings,
    stream: repo::stream::Model,
) -> Result<(), Error> {
    let messages = repo::find_streams_messages_by_stream_id(db, stream.id).await?;

    let request = SummarizeStreamRequest {
        stream_id: Some(stream.id.into()),
        messages: messages
            .iter()
            .map(|message| StreamMessage {
                message_id: Some(message.id.into()),
                user_id: Some(message.user_id.into()),
            })
            .collect(),
        version: Some(Utc::now().timestamp_millis()),
    };

    let mut buf = BytesMut::new();
    request.encode(&mut buf)?;

    js.publish(settings.streams.messaging.subjects.request, buf.into())
        .await?;

    println!("SEND ASYNC");

    Ok(())
}
