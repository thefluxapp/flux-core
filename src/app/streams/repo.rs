use anyhow::Error;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

pub mod stream;
pub mod stream_user;

pub async fn find_last_streams<T: ConnectionTrait>(db: &T) -> Result<Vec<stream::Model>, Error> {
    let streams = stream::Entity::find()
        .filter(stream::Column::IsMain.eq(true))
        // .filter(stream::Column::Text.is_not_null())
        .all(db)
        .await?;

    Ok(streams)
}

pub async fn find_user_streams_with_streams_users<T: ConnectionTrait>(
    db: &T,
    user_id: Uuid,
) -> Result<Vec<stream::Model>, Error> {
    let streams = stream::Entity::find()
        .join(
            JoinType::LeftJoin,
            stream::Entity::belongs_to(stream_user::Entity)
                .to(stream_user::Column::StreamId)
                .from(stream::Column::Id)
                .into(),
        )
        .filter(stream_user::Column::UserId.eq(user_id))
        .order_by_desc(stream::Column::Id)
        .all(db)
        .await?;

    Ok(streams)
}

pub async fn find_streams_with_streams_users<T: ConnectionTrait>(
    db: &T,
    stream_ids: Vec<Uuid>,
) -> Result<Vec<(stream::Model, Vec<stream_user::Model>)>, Error> {
    let streams_users = stream::Entity::find()
        .select_with(stream_user::Entity)
        .join(
            JoinType::LeftJoin,
            stream::Entity::belongs_to(stream_user::Entity)
                .to(stream_user::Column::StreamId)
                .from(stream::Column::Id)
                .into(),
        )
        .filter(stream::Column::Id.is_in(stream_ids))
        // TODO: save original order
        // .order_by_asc(stream::Column::Id)
        .all(db)
        .await?;

    Ok(streams_users)
}

pub async fn update_stream_text<T: ConnectionTrait>(
    db: &T,
    stream_id: Uuid,
    text: String,
    version: DateTime<Utc>,
) -> Result<(), Error> {
    if let Some(stream) = stream::Entity::find_by_id(stream_id)
        .filter(stream::Column::UpdatedAt.lt(version))
        .lock_exclusive()
        .one(db)
        .await?
    {
        let mut stream: stream::ActiveModel = stream.into();
        stream.text = Set(text);
        stream.update(db).await?;
    };

    Ok(())
}
