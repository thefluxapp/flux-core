use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Streams::Table)
                    .col(uuid(Streams::Id).primary_key())
                    .col(text_null(Streams::Title))
                    .col(text_null(Streams::Text))
                    .col(uuid(Streams::MessageId))
                    .col(boolean(Streams::IsMain))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("streams_message_id_udx")
                    .unique()
                    .table(Streams::Table)
                    .col(Streams::MessageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Streams::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Streams {
    Table,
    Id,
    Title,
    Text,
    MessageId,
    IsMain,
}
