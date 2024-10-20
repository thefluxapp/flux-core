use anyhow::Error;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

pub mod stream;

pub async fn find_streams<T: ConnectionTrait>(db: &T) -> Result<Vec<stream::Model>, Error> {
    let streams = stream::Entity::find()
        .order_by_asc(stream::Column::Id)
        .all(db)
        .await?;

    Ok(streams)
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
        stream.text = Set(Some(text));
        stream.update(db).await?;
    };

    Ok(())
}
