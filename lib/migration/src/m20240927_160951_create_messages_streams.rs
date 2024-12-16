use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(MessagesStreams::Table)
                    .col(uuid(MessagesStreams::Id).primary_key())
                    .col(uuid(MessagesStreams::MessageId))
                    .col(uuid(MessagesStreams::StreamId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("messages_streams_message_id_stream_id_udx")
                    .unique()
                    .table(MessagesStreams::Table)
                    .col(MessagesStreams::MessageId)
                    .col(MessagesStreams::StreamId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("messages_streams_message_id_udx")
                    .unique()
                    .table(MessagesStreams::Table)
                    .col(MessagesStreams::MessageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessagesStreams::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum MessagesStreams {
    Table,
    Id,
    MessageId,
    StreamId,
}
