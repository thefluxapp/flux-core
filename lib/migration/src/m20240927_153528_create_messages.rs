use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Messages::Table)
                    .col(uuid(Messages::Id).primary_key())
                    .col(uuid(Messages::UserId))
                    .col(text(Messages::Text))
                    .col(text(Messages::Code))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("messages_code_udx")
                    .unique()
                    .table(Messages::Table)
                    .col(Messages::Code)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Messages {
    Table,
    Id,
    Text,
    UserId,
    Code,
}
