use entity::product::{ActiveModel as ProductActiveModel, Entity as ProductEntity};
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    response::Responder,
    Request, Response,
};
use sea_orm::{
    entity::prelude::*, query::Condition, ActiveModelTrait, ActiveValue, DatabaseConnection,
};
use serde_json::json;
use thiserror::Error;

use crate::{
    db::establish_connection,
    models::{
        product::{ProductDetails, ProductReturn},
        user::{AuthUser, MinUserReturnDto},
    },
};

#[derive(Error, Debug)]
pub enum ProductServiceError {
    #[error(transparent)]
    DbError(crate::db::DbError),
    #[error(transparent)]
    OrmError(sea_orm::DbErr),
    #[error("Product with id {0} not found")]
    NotFound(i64),
    #[error("You are not authorized to perform changes on this product")]
    NotAllowed,
    #[error("An unknown error occurred")]
    Unknown,
}

impl<'r> Responder<'r, 'static> for ProductServiceError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::DbError(_) | Self::OrmError(_) | Self::Unknown => {
                Response::build().status(Status::InternalServerError).ok()
            }
            Self::NotFound(_) => {
                Response::build_from(json!({ "error": format!("{self}") }).respond_to(request)?)
                    .status(Status::NotFound)
                    .ok()
            }
            Self::NotAllowed => {
                Response::build_from(json!({ "error": format!("{self}") }).respond_to(request)?)
                    .status(Status::Forbidden)
                    .ok()
            }
        }
    }
}

pub struct ProductService {
    db_connection: DatabaseConnection,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ProductService {
    type Error = ProductServiceError;

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match establish_connection()
            .await
            .map_err(|e| ProductServiceError::DbError(e))
        {
            Err(e) => return Outcome::Failure((Status::InternalServerError, e)),
            Ok(db) => return Outcome::Success(Self { db_connection: db }),
        }
    }
}

impl ProductService {
    pub async fn create_new_product(
        &mut self,
        create: ProductDetails,
        creating_user: AuthUser,
    ) -> Result<i64, ProductServiceError> {
        let to_create = ProductActiveModel {
            price: ActiveValue::Set(create.price),
            description: ActiveValue::Set(create.description),
            product_title: ActiveValue::Set(create.title),
            created_by: ActiveValue::Set(creating_user.user.id),
            location_city: ActiveValue::Set(create.city),
            location_state: ActiveValue::Set(create.state),
            location_country: ActiveValue::Set(create.country),
            location_latitude: ActiveValue::Set(create.latitude),
            location_longitude: ActiveValue::Set(create.longitude),
            location_zip: ActiveValue::Set(create.zip),
            ..Default::default()
        };

        let created = to_create
            .insert(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        Ok(created.id)
    }

    pub async fn get_product_by_id(
        &mut self,
        id: i64,
    ) -> Result<ProductReturn, ProductServiceError> {
        use entity::product;
        let found = ProductEntity::find()
            .find_also_related(entity::user::Entity)
            .filter(Condition::all().add(product::Column::Id.eq(id)))
            .one(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        if let Some((prod, user)) = found {
            if let None = user {
                return Err(ProductServiceError::Unknown);
            }
            let user = user.unwrap();
            return Ok(ProductReturn {
                title: prod.product_title,
                description: prod.description,
                price: f64::try_from(prod.price).map_err(|_| ProductServiceError::Unknown)?,
                created_by: MinUserReturnDto {
                    id: user.id,
                    username: user.username,
                },
            });
        } else {
            return Err(ProductServiceError::NotFound(id));
        }
    }

    pub async fn update_product_by_id(
        &mut self,
        id: i64,
        product: ProductDetails,
        user: AuthUser,
    ) -> Result<(), ProductServiceError> {
        let db_product = self.get_product_by_id(id).await?;
        if db_product.created_by.id != user.user.id {
            return Err(ProductServiceError::NotAllowed);
        }

        let active_product: entity::product::ActiveModel = ProductEntity::find()
            .filter(Condition::all().add(entity::product::Column::Id.eq(id)))
            .one(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?
            .ok_or(ProductServiceError::NotFound(id))?
            .into();

        let new_prod = entity::product::ActiveModel {
            description: ActiveValue::Set(product.description),
            location_city: ActiveValue::Set(product.city),
            location_country: ActiveValue::Set(product.country),
            location_state: ActiveValue::Set(product.state),
            location_zip: ActiveValue::Set(product.zip),
            location_latitude: ActiveValue::Set(product.latitude),
            location_longitude: ActiveValue::Set(product.longitude),
            price: ActiveValue::Set(product.price),
            product_title: ActiveValue::Set(product.title),
            ..active_product
        };

        new_prod
            .update(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        Ok(())
    }

    pub async fn delete_product_by_id(
        &mut self,
        id: i64,
        user: AuthUser,
    ) -> Result<(), ProductServiceError> {
        let product: ProductActiveModel = ProductEntity::find_by_id(id)
            .one(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?
            .ok_or(ProductServiceError::NotFound(id))?
            .into();

        if product.created_by.clone().unwrap() != user.user.id {
            return Err(ProductServiceError::NotAllowed);
        }

        product
            .delete(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        Ok(())
    }
}
