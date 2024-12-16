use anyhow::Error;
use flux_core_api::{
    messages_service_server::MessagesService, CreateMessageRequest, CreateMessageResponse,
    GetMessageRequest, GetMessageResponse,
};
use tonic::{Request, Response, Status};

use crate::app::{error::AppError, state::AppState};

use super::{repo, service};

pub struct GrpcMessagesService {
    pub state: AppState,
}

impl GrpcMessagesService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl MessagesService for GrpcMessagesService {
    async fn create_message(
        &self,
        request: Request<CreateMessageRequest>,
    ) -> Result<Response<CreateMessageResponse>, Status> {
        let response = create_message(&self.state, request.into_inner()).await?;

        Ok(Response::new(response.into()))
    }

    async fn get_message(
        &self,
        request: Request<GetMessageRequest>,
    ) -> Result<Response<GetMessageResponse>, Status> {
        let response = get_message(&self.state, request.into_inner()).await?;

        Ok(Response::new(response.into()))
    }
}

async fn get_message(
    state: &AppState,
    request: GetMessageRequest,
) -> Result<service::get_message::Response, AppError> {
    let response = service::get_message(&state.db, request.try_into()?).await?;

    Ok(response)
}

mod get_message {
    use flux_core_api::{get_message_response::Message, GetMessageRequest, GetMessageResponse};
    use prost_types::Timestamp;
    use uuid::Uuid;
    use validator::ValidationErrors;

    use crate::app::{
        error::AppError,
        messages::{
            repo::{message, stream},
            service::get_message::{Request, Response},
        },
    };

    impl TryFrom<GetMessageRequest> for Request {
        type Error = AppError;

        fn try_from(req: GetMessageRequest) -> Result<Self, Self::Error> {
            Ok(Self {
                message_id: Uuid::parse_str(req.message_id())
                    .map_err(|_| AppError::Validation(ValidationErrors::new()))?,
            })
        }
    }

    impl From<Response> for GetMessageResponse {
        fn from(res: Response) -> Self {
            Self {
                message: Some(M(res.message.0, res.message.1).into()),
                messages: res
                    .messages
                    .into_iter()
                    .map(|message| M(message.0, message.1).into())
                    .collect(),
            }
        }
    }

    struct M(message::Model, Option<stream::Model>);

    impl From<M> for Message {
        fn from(M(message, stream): M) -> Self {
            Self {
                message_id: Some(message.id.to_string()),
                user_id: Some(message.user_id.to_string()),
                text: Some(message.text),
                code: Some(message.code),
                stream_id: match stream {
                    Some(stream) => Some(stream.id.to_string()),
                    None => None,
                },
                order: Some(message.created_at.and_utc().timestamp_micros()),
                created_at: Some(Timestamp {
                    seconds: message.created_at.and_utc().timestamp(),
                    nanos: 0,
                }),
            }
        }
    }
}

async fn create_message(
    state: &AppState,
    request: CreateMessageRequest,
) -> Result<service::create_message::Response, AppError> {
    let response = service::create_message(&state.db, request.try_into()?).await?;

    if let Some(ref stream) = response.stream {
        tokio::spawn(summarize_stream_by_message_id(
            state.clone(),
            stream.clone(),
        ));
    }

    tokio::spawn(notify_message(
        state.clone(),
        service::notify_message::Req {
            message: response.message.clone(),
            stream: response.stream.clone(),
        },
    ));

    Ok(response)
}

mod create_message {
    use flux_core_api::{CreateMessageRequest, CreateMessageResponse};
    use uuid::Uuid;
    use validator::{Validate as _, ValidationErrors};

    use crate::app::{error::AppError, messages::service};

    impl TryFrom<CreateMessageRequest> for service::create_message::Request {
        type Error = AppError;

        fn try_from(request: CreateMessageRequest) -> Result<Self, Self::Error> {
            let data = Self {
                text: request.text().into(),
                code: request.code().into(),
                user_id: Uuid::parse_str(request.user_id())
                    .map_err(|_| AppError::Validation(ValidationErrors::new()))?,
                message_id: match request.message_id {
                    Some(message_id) => Some(
                        Uuid::parse_str(&message_id)
                            .map_err(|_| AppError::Validation(ValidationErrors::new()))?,
                    ),
                    None => None,
                },
            };
            data.validate()?;

            Ok(data)
        }
    }

    impl From<service::create_message::Response> for CreateMessageResponse {
        fn from(val: service::create_message::Response) -> Self {
            Self {
                message_id: Some(val.message.id.into()),
            }
        }
    }
}

async fn notify_message(
    AppState { js, settings, .. }: AppState,
    req: service::notify_message::Req,
) -> Result<(), Error> {
    service::notify_message(&js, settings.messages.messaging, req).await?;

    Ok(())
}

async fn summarize_stream_by_message_id(
    AppState { settings, db, js }: AppState,
    stream: repo::stream::Model,
) -> Result<(), AppError> {
    service::summarize_stream_by_message_id(&db, &js, settings, stream).await?;

    Ok(())
}
