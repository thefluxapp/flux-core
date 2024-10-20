use flux_core_api::{
    streams_service_server::StreamsService, GetStreamsRequest, GetStreamsResponse,
};
use tonic::{Request, Response, Status};

use super::{repo, service};
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
        _: Request<GetStreamsRequest>,
    ) -> Result<Response<GetStreamsResponse>, Status> {
        let response = get_streams(&self.state).await?;

        Ok(Response::new(response))
    }
}

async fn get_streams(AppState { db, .. }: &AppState) -> Result<GetStreamsResponse, AppError> {
    let response = service::get_streams(db).await?;

    Ok(response.into())
}

impl From<service::GetStreamsResponse> for GetStreamsResponse {
    fn from(response: service::GetStreamsResponse) -> Self {
        GetStreamsResponse {
            streams: response
                .streams
                .into_iter()
                .map(|stream| stream.into())
                .collect(),
        }
    }
}

impl From<repo::stream::Model> for flux_core_api::get_streams_response::Stream {
    fn from(stream: repo::stream::Model) -> Self {
        Self {
            stream_id: Some(stream.id.into()),
            message_id: Some(stream.message_id.into()),
            text: stream.text,
        }
    }
}
