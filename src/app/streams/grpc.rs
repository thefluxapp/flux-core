use flux_core_api::{
    streams_service_server::StreamsService, GetLastStreamsRequest, GetLastStreamsResponse,
    GetStreamsRequest, GetStreamsResponse, GetUserStreamsRequest, GetUserStreamsResponse,
};
use tonic::{Request, Response, Status};

use super::service;
use crate::app::{error::AppError, state::AppState};

pub struct GrpcStreamsService {
    pub state: AppState,
}

impl GrpcStreamsService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl StreamsService for GrpcStreamsService {
    async fn get_streams(
        &self,
        req: Request<GetStreamsRequest>,
    ) -> Result<Response<GetStreamsResponse>, Status> {
        let response = get_streams(&self.state, req.into_inner()).await?;

        Ok(Response::new(response))
    }

    async fn get_last_streams(
        &self,
        _: Request<GetLastStreamsRequest>,
    ) -> Result<Response<GetLastStreamsResponse>, Status> {
        let response = get_last_streams(&self.state).await?;

        Ok(Response::new(response))
    }

    async fn get_user_streams(
        &self,
        req: Request<GetUserStreamsRequest>,
    ) -> Result<Response<GetUserStreamsResponse>, Status> {
        let response = get_user_streams(&self.state, req.into_inner()).await?;

        Ok(Response::new(response))
    }
}

async fn get_streams(
    AppState { db, .. }: &AppState,
    req: GetStreamsRequest,
) -> Result<GetStreamsResponse, AppError> {
    let res = service::get_streams(db, req.try_into()?).await?;

    Ok(res.into())
}

mod get_streams {
    use anyhow::Error;
    use flux_core_api::{GetStreamsRequest, GetStreamsResponse};
    use uuid::Uuid;

    use crate::app::{
        error::AppError,
        streams::{
            repo::{stream, stream_user},
            service::get_streams::{Req, Res},
        },
    };

    impl TryFrom<GetStreamsRequest> for Req {
        type Error = AppError;

        fn try_from(req: GetStreamsRequest) -> Result<Self, Self::Error> {
            let data = Self {
                stream_ids: req
                    .stream_ids
                    .iter()
                    .map(|id| -> Result<Uuid, Error> { Ok(Uuid::parse_str(id)?) })
                    .collect::<Result<Vec<Uuid>, Error>>()?,
            };

            Ok(data)
        }
    }

    impl From<Res> for GetStreamsResponse {
        fn from(response: Res) -> Self {
            Self {
                streams: response
                    .streams
                    .into_iter()
                    .rev()
                    .map(|(stream, streams_users)| U(stream, streams_users).into())
                    .collect(),
            }
        }
    }

    struct U(stream::Model, Vec<stream_user::Model>);

    impl From<U> for flux_core_api::get_streams_response::Stream {
        fn from(U(stream, streams_users): U) -> Self {
            Self {
                stream_id: Some(stream.id.into()),
                message_id: Some(stream.message_id.into()),
                text: stream.text,
                user_ids: streams_users.iter().map(|m| m.user_id.into()).collect(),
            }
        }
    }
}

async fn get_last_streams(
    AppState { db, .. }: &AppState,
) -> Result<GetLastStreamsResponse, AppError> {
    let res = service::get_last_streams(db).await?;

    Ok(res.into())
}

mod get_last_streams {
    use flux_core_api::GetLastStreamsResponse;
    use itertools::Itertools as _;

    use crate::app::streams::service::get_last_streams::Res;

    impl From<Res> for GetLastStreamsResponse {
        fn from(req: Res) -> Self {
            Self {
                stream_ids: req.stream_ids.into_iter().map_into().collect(),
            }
        }
    }
}

async fn get_user_streams(
    AppState { db, .. }: &AppState,
    req: GetUserStreamsRequest,
) -> Result<GetUserStreamsResponse, AppError> {
    let res = service::get_user_streams(db, req.try_into()?).await?;

    Ok(res.into())
}

mod get_user_streams {
    use flux_core_api::{GetUserStreamsRequest, GetUserStreamsResponse};
    use itertools::Itertools;

    use crate::app::{
        error::AppError,
        streams::service::get_user_streams::{Req, Res},
    };

    impl TryFrom<GetUserStreamsRequest> for Req {
        type Error = AppError;

        fn try_from(req: GetUserStreamsRequest) -> Result<Self, Self::Error> {
            let data = Self {
                user_id: req.user_id().try_into()?,
            };

            Ok(data)
        }
    }

    impl From<Res> for GetUserStreamsResponse {
        fn from(res: Res) -> Self {
            Self {
                stream_ids: res.stream_ids.into_iter().map_into().collect(),
            }
        }
    }
}
