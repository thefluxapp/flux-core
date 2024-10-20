use anyhow::Error;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ColumnTrait as _, ConnectionTrait, EntityTrait as _,
    IntoActiveModel as _, Order, QueryFilter as _, QueryOrder,
};
use uuid::Uuid;

pub mod message;
pub mod message_stream;
pub mod stream;
pub mod stream_user;

pub async fn create_message<T: ConnectionTrait>(
    db: &T,
    model: message::Model,
) -> Result<message::Model, Error> {
    let message = model.into_active_model().insert(db).await?;

    Ok(message)
}

pub async fn find_message_by_id<T: ConnectionTrait>(
    db: &T,
    message_id: Uuid,
) -> Result<Option<message::Model>, Error> {
    let message = message::Entity::find_by_id(message_id).one(db).await?;

    Ok(message)
}

pub async fn create_stream<T: ConnectionTrait>(
    db: &T,
    model: stream::Model,
) -> Result<Option<stream::Model>, Error> {
    let message_id = model.message_id;

    stream::Entity::insert(model.into_active_model())
        .on_conflict(
            OnConflict::column(stream::Column::MessageId)
                .do_nothing()
                .to_owned(),
        )
        .do_nothing()
        .exec(db)
        .await?;

    let stream = stream::Entity::find()
        .filter(stream::Column::MessageId.eq(message_id))
        .one(db)
        .await?;

    Ok(stream)
}

pub async fn create_message_stream<T: ConnectionTrait>(
    db: &T,
    model: message_stream::Model,
) -> Result<(), Error> {
    message_stream::Entity::insert(model.into_active_model())
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .do_nothing()
        .exec(db)
        .await?;
    Ok(())
}

pub async fn create_stream_user<T: ConnectionTrait>(
    db: &T,
    model: stream_user::Model,
) -> Result<(), Error> {
    stream_user::Entity::insert(model.into_active_model())
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .do_nothing()
        .exec(db)
        .await?;

    Ok(())
}

pub async fn find_streams_messages_by_stream_id<T: ConnectionTrait>(
    db: &T,
    stream_id: Uuid,
) -> Result<Vec<message::Model>, Error> {
    let messages = message::Entity::find()
        .inner_join(message_stream::Entity)
        .filter(message_stream::Column::StreamId.eq(stream_id))
        .order_by(message::Column::Id, Order::Asc)
        .all(db)
        .await?;

    Ok(messages)
}

pub async fn find_messages_by_ids<T: ConnectionTrait>(
    db: &T,
    message_ids: Vec<Uuid>,
) -> Result<Vec<message::Model>, Error> {
    let messages = message::Entity::find()
        .filter(message::Column::Id.is_in(message_ids))
        .order_by(message::Column::Id, Order::Asc)
        .all(db)
        .await?;

    Ok(messages)
}

pub async fn find_stream_by_message_id<T: ConnectionTrait>(
    db: &T,
    stream_id: Uuid,
) -> Result<Option<stream::Model>, Error> {
    let message = stream::Entity::find()
        .filter(stream::Column::MessageId.eq(stream_id))
        .one(db)
        .await?;

    Ok(message)
}

pub async fn find_messages_by_stream_id<T: ConnectionTrait>(
    db: &T,
    stream_id: Uuid,
) -> Result<Vec<message_stream::Model>, Error> {
    Ok(message_stream::Entity::find()
        .filter(message_stream::Column::StreamId.eq(stream_id))
        .order_by_asc(message_stream::Column::MessageId)
        .all(db)
        .await?)
}
