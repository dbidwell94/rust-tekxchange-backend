use super::user::MinUserReturnDto;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductDetails {
    pub description: String,
    pub title: String,
    pub price: Decimal,
    pub country: String,
    pub state: String,
    pub city: String,
    pub zip: String,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductReturn {
    pub title: String,
    pub description: String,
    pub price: f64,
    pub created_by: MinUserReturnDto,
}
