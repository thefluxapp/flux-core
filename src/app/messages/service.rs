use crate::app::{error::AppError, settings::AppSettings, AppJS};
use anyhow::Error;
use bytes::BytesMut;
use chrono::Utc;
use create_message::{Request, Response};
use flux_core_api::{summarize_stream_request::Message as StreamMessage, SummarizeStreamRequest};
use prost::Message;
use sea_orm::{DbConn, TransactionTrait as _};
use uuid::Uuid;

use super::{repo, settings::MessagingSettings};

pub async fn get_message(
    db: &DbConn,
    req: get_message::Request,
) -> Result<get_message::Response, Error> {
    let message = repo::find_message_by_id(db, req.message_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let stream = repo::find_stream_by_message_id(db, req.message_id).await?;
    let parent_stream = match stream {
        Some(ref stream) => {
            repo::find_parent_stream_by_message_id(db, message.id, stream.id).await?
        }
        None => None,
    };

    let message = (message, parent_stream);

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
            code: request.code,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        },
    )
    .await?;

    let stream = match request.message_id {
        Some(message_id) => {
            let parent_message = repo::find_message_by_id(&txn, message_id)
                .await?
                .ok_or(AppError::NotFound)?;

            let is_main = match repo::find_message_stream_by_message_id(&txn, message_id).await? {
                Some(_) => false,
                None => true,
            };

            let stream = repo::create_stream(
                &txn,
                repo::stream::Model {
                    id: Uuid::now_v7(),
                    title: None,
                    text: None,
                    message_id,
                    is_main,
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

            repo::create_stream_user(
                &txn,
                repo::stream_user::Model {
                    id: Uuid::now_v7(),
                    stream_id: stream.id,
                    user_id: parent_message.user_id,
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
        pub code: String,
    }

    pub struct Response {
        pub message: message::Model,
        pub stream: Option<stream::Model>,
    }
}

pub async fn notify_message(
    // db: &DbConn,
    js: &AppJS,
    settings: MessagingSettings,
    req: notify_message::Req,
) -> Result<(), Error> {
    let mut buf = BytesMut::new();
    Into::<flux_core_api::Message>::into(req).encode(&mut buf)?;

    js.publish(settings.message.subject, buf.into()).await?;

    Ok(())
}

pub mod notify_message {
    use flux_core_api::message::{Message, Stream};
    use prost_types::Timestamp;

    use crate::app::messages::repo;

    pub struct Req {
        pub message: repo::message::Model,
        pub stream: Option<repo::stream::Model>,
    }

    impl From<Req> for flux_core_api::Message {
        fn from(Req { message, stream }: Req) -> Self {
            Self {
                message: Some(Message {
                    message_id: Some(message.id.into()),
                    user_id: Some(message.id.into()),
                    text: Some(message.text),
                    code: Some(message.code),
                    order: Some(message.created_at.and_utc().timestamp_micros()),
                    created_at: Some(Timestamp {
                        seconds: message.created_at.and_utc().timestamp(),
                        nanos: 0,
                    }),
                    updated_at: Some(Timestamp {
                        seconds: message.updated_at.and_utc().timestamp(),
                        nanos: 0,
                    }),
                }),
                stream: match stream {
                    Some(stream) => Some(Stream {
                        stream_id: Some(stream.id.into()),
                        message_id: Some(stream.message_id.into()),
                    }),
                    None => None,
                },
            }
        }
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

    Ok(())
}
