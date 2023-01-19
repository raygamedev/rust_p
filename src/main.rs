mod api;
mod db;
mod models;

#[macro_use]
extern crate rocket;
extern crate core;

use crate::api::{card, merchant, user_api};
use db::mongo::Mongo;
use env_logger::Builder;

use log::LevelFilter;
use rocket::Build;
use std::io::Write;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> rocket::Rocket<Build> {
    if Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .try_init()
        .is_ok()
    {}

    let mongo = Mongo::init("users").await;
    rocket::build()
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![
                user_api::new_user,
                user_api::auth_user,
                merchant::new_merchant,
                merchant::get_merchant,
                merchant::get_all_merchants,
                card::new_card,
                card::active_card,
                card::add_mark
            ],
        )
        .manage(mongo)
}
