use crate::db::mongo::Mongo;
use crate::models::models::Merchant;
use mongodb::bson::oid::ObjectId;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use std::str::FromStr;

#[post("/new_merchant", data = "<merchant>")]
pub async fn new_merchant(mongo: &State<Mongo>, merchant: Json<Merchant>) -> Status {
    match mongo.insert_merchant(merchant.into_inner()).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

#[get("/merchant?<merchant_id>")]
pub async fn get_merchant(
    mongo: &State<Mongo>,
    merchant_id: String,
) -> Result<Json<Merchant>, Status> {
    match ObjectId::from_str(&merchant_id) {
        Ok(oid) => match mongo.get_merchant(oid).await {
            Some(merchant) => Ok(Json(merchant)),
            None => Err(Status::NotImplemented),
        },
        Err(_) => Err(Status::BadRequest),
    }
}
#[get("/merchants")]
pub async fn get_all_merchants(mongo: &State<Mongo>) -> Result<Json<Vec<Merchant>>, Status> {
    let mongo_entries = mongo.get_all_merchants().await;
    let merchants: Vec<Merchant> = mongo_entries
        .into_iter()
        .map(|entry| entry.unwrap())
        .collect();
    Ok(Json(merchants))
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::serde::json::serde_json;
    use rocket::tokio;

    use crate::rocket;
    #[tokio::test]
    async fn test_add_merchant() {
        let client = Client::debug(rocket().await).await.unwrap();
        let merchant = Merchant {
            _id: ObjectId::new(),
            name: "test_merchant".to_string(),
            address: "".to_string(),
            logo: "".to_string(),
        };
        let res = client
            .post("/api/new_merchant")
            .body(serde_json::to_string(&merchant).unwrap())
            .dispatch()
            .await;
        info!("{:?}", res);
        assert_eq!(res.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_get_non_existing_merchant() {
        let client = Client::debug(rocket().await).await.unwrap();
        let merchant_id = ObjectId::new().to_string();
        let res = client
            .get(format!("/api/merchant?merchant_id={merchant_id}"))
            .dispatch()
            .await;
        info!("{:?}", res);
        assert_eq!(res.status(), Status::NotImplemented);
    }
    #[tokio::test]
    async fn test_get_existing_merchant() {
        let client = Client::debug(rocket().await).await.unwrap();
        let merchant = Merchant {
            _id: ObjectId::new(),
            name: "Nahat Coffee".to_string(),
            address: "Dizengoff Square 1, Tel Aviv-Yafo".to_string(),
            logo: "".to_string(),
        };
        let res = client
            .post("/api/new_merchant")
            .body(serde_json::to_string(&merchant).unwrap())
            .dispatch()
            .await;
        info!("{:?}", res);
        assert_eq!(res.status(), Status::Ok);
        let res = client
            .get(format!("/api/merchant?merchant_id={}", merchant._id))
            .dispatch()
            .await;
        info!("{:?}", res);
        assert_eq!(res.status(), Status::Ok);
    }
    #[tokio::test]
    async fn test_get_all_merchants() {
        let client = Client::debug(rocket().await).await.unwrap();
        let res = client.get("/api/merchants").dispatch().await;
        assert_eq!(res.status(), Status::Ok);
    }
}
