use crate::db::mongo::Mongo;
use crate::models::models::User;
use mongodb::bson::Uuid;
use rocket::http::Status;
use rocket::State;
use std::collections::HashMap;

#[post("/new_user?<user_id>")]
pub async fn new_user(mongo: &State<Mongo>, user_id: String) -> Status {
    match Uuid::parse_str(&user_id) {
        Ok(uuid) => {
            let user = User {
                _id: uuid,
                cards: HashMap::new(),
            };
            match mongo.insert_user(user, Some("users")).await {
                Ok(_) => Status::Ok,
                Err(_) => Status::InternalServerError,
            }
        }
        Err(_) => Status::BadRequest,
    }
}

#[get("/auth?<user_id>")]
pub async fn auth_user(mongo: &State<Mongo>, user_id: String) -> Status {
    info!("authenticating user {}", user_id);
    match Uuid::parse_str(&user_id) {
        Ok(uuid) => match mongo.get_user(uuid, None).await {
            Ok(user) => {
                info!("user {:?} authenticated", user);
                match user {
                    Some(_) => Status::Ok,
                    None => Status::InternalServerError,
                }
            }
            Err(_) => Status::InternalServerError,
        },
        Err(_) => Status::BadRequest,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rocket;
    use rocket::local::asynchronous::Client;
    use rocket::tokio;

    #[tokio::test]
    async fn test_new_user_api() {
        let client = Client::debug(rocket().await).await.unwrap();
        let user_id = Uuid::new().to_string();
        let res = client
            .post(format!("/api/new_user?user_id={user_id}"))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_auth_user() {
        let client = Client::debug(rocket().await).await.unwrap();
        let user_id = Uuid::new().to_string();
        let res = client
            .post(format!("/api/new_user?user_id={user_id}"))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Ok);
        let res = client
            .get(format!("/api/auth?user_id={user_id}"))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Ok);
    }
}
