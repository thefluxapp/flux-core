use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub text: String,
    pub code: String,
    pub user_id: Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// impl Model {
//     pub fn status(&self) -> String {
//         String::from("active")
//     }

//     pub fn order(&self) -> i64 {
//         self.created_at.and_utc().timestamp_micros()
//     }
// }

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // #[sea_orm(
    //     belongs_to = "super::user::Entity",
    //     from = "Column::UserId",
    //     to = "super::user::Column::Id"
    // )]
    // User,
    #[sea_orm(has_one = "super::message_stream::Entity")]
    MessageStream,
    // #[sea_orm(has_one = "super::stream::Entity")]
    // Stream,
}

impl Related<super::message_stream::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MessageStream.def()
    }
}

// impl Related<super::stream::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::Stream.def()
//     }
// }

// impl Related<super::user::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::User.def()
//     }
// }

// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
// pub enum Relation {}
