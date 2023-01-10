//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "category")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(unique)]
    pub category_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_category::Entity")]
    ProductCategory,
}

impl Related<super::product_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductCategory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
