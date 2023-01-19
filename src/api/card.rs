use crate::db::mongo::Mongo;
use crate::models::models::{CardMark, CardModel, Color, AVAILABLE_CARD_COLORS};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Uuid;
use rand::Rng;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

fn generate_card_id() -> String {
    let mut rng = rand::thread_rng();
    let six_digit_number: u32 = rng.gen_range(100000..999999);
    six_digit_number.to_string()
}

fn get_random_color() -> Color {
    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(0..AVAILABLE_CARD_COLORS.len());
    AVAILABLE_CARD_COLORS[random_num].clone()
}

fn deserialize_req_params(
    used_id: String,
    merchant_id: String,
) -> Result<(Uuid, ObjectId), anyhow::Error> {
    info!("deserializing user_id to UUID: {}", used_id);
    info!("deserializing merchant_id to ObjectId: {}", merchant_id);
    let user_id = Uuid::parse_str(used_id)?;
    let merchant_id: ObjectId = merchant_id.parse()?;
    Ok((user_id, merchant_id))
}

#[put("/new_card?<user_id>&<merchant_id>")]
pub async fn new_card(
    mongo: &State<Mongo>,
    user_id: String,
    merchant_id: String,
) -> Result<Json<CardModel>, Status> {
    let (user_uuid, merchant_oid) = match deserialize_req_params(user_id, merchant_id) {
        Err(_) => return Err(Status::InternalServerError),
        Ok((u, m)) => (u, m),
    };

    info!(
        "creating new card for user {}, merchant: {}",
        user_uuid, merchant_oid
    );

    let card: CardModel = CardModel {
        marks: vec![],
        color: get_random_color(),
        card_id: generate_card_id(),
        merchant_id: merchant_oid.to_string(),
    };
    let res = mongo
        .update_user_cards(user_uuid, &card, merchant_oid, Some("users"))
        .await;
    match res {
        Ok(_) => Ok(Json(card)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/active_card?<user_id>&<merchant_id>")]
pub async fn active_card(
    mongo: &State<Mongo>,
    user_id: String,
    merchant_id: String,
) -> Result<Json<CardModel>, Status> {
    let (user_uuid, merchant_oid) = match deserialize_req_params(user_id, merchant_id) {
        Err(_) => return Err(Status::InternalServerError),
        Ok((u, m)) => (u, m),
    };
    let card: Result<CardModel, anyhow::Error> =
        mongo.get_user_card(user_uuid, merchant_oid, None).await;
    match card {
        Ok(c) => Ok(Json(c)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[put("/add_mark?<user_id>&<merchant_id>&<mark_index>")]
pub async fn add_mark(
    mongo: &State<Mongo>,
    user_id: String,
    merchant_id: String,
    mark_index: i32,
) -> Result<Json<CardModel>, Status> {
    let (user_uuid, merchant_oid) = match deserialize_req_params(user_id, merchant_id) {
        Err(_) => return Err(Status::InternalServerError),
        Ok((u, m)) => (u, m),
    };
    let user = match mongo.get_user(user_uuid, Some("users")).await {
        Err(_) => return Err(Status::InternalServerError),
        Ok(u) => u,
    };

    let mut user = match user {
        None => return Err(Status::InternalServerError),
        Some(user) => user,
    };

    match user.cards.get_mut(&merchant_oid) {
        None => Err(Status::InternalServerError),
        Some(card) => {
            card.marks.push(CardMark {
                index: mark_index,
                is_marked: true,
            });
            info!("{:#?}", card);
            Ok(Json(card.clone()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_card_id() {
        let card_id = generate_card_id();
        info!("{}", card_id);
        assert_eq!(card_id.len(), 6);
    }
}
