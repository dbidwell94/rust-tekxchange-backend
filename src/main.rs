#[macro_use]
extern crate rocket;
mod controllers;
mod db;
mod models;
mod services;
use migration::{Migrator, MigratorTrait};
use services::UserService;
use std::env;

use crate::models::user::UserRegister;

#[launch]
pub async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let conn = db::establish_connection().await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let mut user_service = UserService::new(conn);
    let found_admin = user_service
        .username_exists(crate::models::user::ADMIN_USERNAME)
        .await
        .unwrap();

    if !found_admin {
        println!("No admin found -- Seeding new admin");
        let admin_email = env::var("ADMIN_EMAIL").unwrap();
        let admin_password = env::var("ADMIN_PASSWORD").unwrap();
        let user_register = UserRegister {
            email: admin_email,
            password: admin_password,
            username: crate::models::user::ADMIN_USERNAME.to_owned(),
        };
        let user_id = user_service.create_user(user_register).await.unwrap();
        user_service
            .update_role_for_user(user_id, models::role::Role::Admin)
            .await
            .unwrap();
    }

    controllers::mount_routes(rocket::build())
}
