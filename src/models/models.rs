use mongodb::bson::oid::ObjectId;
use mongodb::bson::uuid::Uuid;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static AVAILABLE_CARD_COLORS: [Color; 9] = [
    Color {
        card_color: 0xFFEAAB7E,
        mark_color: 0xFFF1C9AD,
    },
    Color {
        card_color: 0xFFA25E58,
        mark_color: 0xFFC2827C,
    },
    Color {
        card_color: 0xFF7F946C,
        mark_color: 0xFFABC296,
    },
    Color {
        card_color: 0xFFCFC4AC,
        mark_color: 0xFFDFD0AE,
    },
    Color {
        card_color: 0xFF9EA779,
        mark_color: 0xFFD2DAB4,
    },
    Color {
        card_color: 0xFF7B9091,
        mark_color: 0xFFA5BABB,
    },
    Color {
        card_color: 0xFFAA9664,
        mark_color: 0xFFC6B99A,
    },
    Color {
        card_color: 0xFFBBA6BE,
        mark_color: 0xFFDDC7E0,
    },
    Color {
        card_color: 0xFFCA9C97,
        mark_color: 0xFFEAC6C2,
    },
];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Color {
    pub card_color: u32,
    pub mark_color: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CardMark {
    pub index: i32,
    pub is_marked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CardModel {
    pub marks: Vec<CardMark>,
    pub color: Color,
    pub card_id: String,
    pub merchant_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Merchant {
    pub _id: ObjectId,
    pub name: String,
    pub address: String,
    pub logo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub _id: Uuid,
    pub cards: HashMap<ObjectId, CardModel>,
}
