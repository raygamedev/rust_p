use crate::models::models::{CardModel, Merchant, User};
use anyhow::anyhow;
use dotenv::Error::__Nonexhaustive;
use log::info;
use mongodb::bson::{doc, to_bson, Uuid};
use mongodb::results::{InsertOneResult, UpdateResult};
use mongodb::{error, Client, Collection, Database};
use std::fmt::format;
use std::{env, io};
// use mongodb::bson::Bson::Binary;
use mongodb::bson::oid::ObjectId;
use mongodb::error::WriteFailure::WriteError;
use mongodb::error::{Error, ErrorKind};
use rocket::futures::StreamExt;

pub struct Mongo {
    db: Database,
}

impl Mongo {
    pub async fn init(db_name: &str) -> Self {
        info!("Creating new Mongo client");
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v,
            Err(_) => "Error loading env variable".to_string(),
        };
        let client = Client::with_uri_str(uri)
            .await
            .expect("Error creating mongo client");

        Self {
            db: client.database(db_name),
        }
    }

    pub async fn insert_user(
        &self,
        user: User,
        col_name: Option<&str>,
    ) -> Result<(), error::Error> {
        let col = match col_name {
            Some(v) => self.db.collection(v),
            None => self.db.collection("users"),
        };
        col.insert_one(user, None).await?;
        Ok(())
    }

    pub async fn get_user(
        &self,
        user_id: Uuid,
        col_name: Option<&str>,
    ) -> Result<Option<User>, error::Error> {
        info!("Getting user with id {}", user_id);
        let col: Collection<User> = match col_name {
            Some(v) => self.db.collection(v),
            None => self.db.collection("users"),
        };
        let res: Option<User> = col.find_one(doc! {"_id": user_id}, None).await?;
        info!("get_user res: {:?}", res);
        Ok(res)
    }

    pub async fn update_user_cards(
        &self,
        user_id: Uuid,
        card: &CardModel,
        merchant_oid: ObjectId,
        col_name: Option<&str>,
    ) -> Result<(), error::Error> {
        info!("Updating user with id {}", user_id);
        let col: Collection<User> = self.db.collection(col_name.unwrap_or("users"));

        let card_bson = to_bson(card)?;
        let filter = doc! {"_id": user_id};
        let update = doc! {"$set": {format!("cards.{merchant_oid}"): card_bson}};
        info!("update_user_cards filter: {:?}", filter);
        info!("update_user_cards update: {:?}", update);

        let res = col.update_one(filter, update, None).await?;
        let UpdateResult { .. } = res;
        {
            info!("Updated user with id {}", user_id);
            Ok(())
        }
    }

    pub async fn get_user_card(
        &self,
        user_id: Uuid,
        merchant_oid: ObjectId,
        col_name: Option<&str>,
    ) -> Result<CardModel, anyhow::Error> {
        info!("Getting user with id {}", user_id);
        let col: Collection<User> = self.db.collection(col_name.unwrap_or("users"));
        let res: Option<User> = col.find_one(doc! {"_id": user_id}, None).await?;
        let user: User = match res {
            None => return Err(anyhow!("user not found")),
            Some(user) => user,
        };
        let card = user.cards.get(&merchant_oid);
        return match card {
            None => Err(anyhow!("Card not found")),
            Some(c) => Ok(c.clone()),
        };
    }

    pub async fn insert_merchant(
        &self,
        merchant: Merchant,
    ) -> Result<InsertOneResult, error::Error> {
        let merchants_col: Collection<Merchant> = self.db.collection("merchants");
        let res: InsertOneResult = merchants_col
            .insert_one(merchant, None)
            .await
            .expect("Error inserting merchant");
        Ok(res)
    }
    pub async fn get_merchant(&self, merchant_id: ObjectId) -> Option<Merchant> {
        let merchants_col: Collection<Merchant> = self.db.collection("merchants");
        let res: Option<Merchant> = merchants_col
            .find_one(doc! {"_id": merchant_id}, None)
            .await
            .expect("Error getting merchant");
        res
    }

    pub async fn get_all_merchants(&self) -> Vec<Option<Merchant>> {
        let merchants_col: Collection<Merchant> = self.db.collection("merchants");
        let cursor = merchants_col.find(None, None).await.unwrap();
        let documents = cursor
            .map(|result| match result {
                Ok(document) => Some(document),
                Err(_) => None,
            })
            .collect::<Vec<Option<Merchant>>>()
            .await;
        documents
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::info;
    use mongodb::bson::oid::ObjectId;
    use rocket::tokio;
    use serial_test::serial;
    use std::collections::HashMap;

    static DB_NAME: &str = "test_db";
    static COL_NAME: &str = "test_users";

    async fn teardown(mongo: Mongo) {
        info!("teardown");
        mongo.db.drop(None).await.unwrap();
    }

    // #[serial]
    // #[tokio::test]
    // async fn test_new() {
    //     let mongo = Mongo::init(DB_NAME).await;
    //     // let res = mongo.client.list_database_names(None, None).await.is_ok();
    //     teardown(mongo).await;
    //     assert!(res);
    // }

    #[serial]
    #[tokio::test]
    async fn test_insert_user() {
        let mongo = Mongo::init(DB_NAME).await;
        let user = User {
            _id: Uuid::new(),
            cards: HashMap::new(),
        };
        let res = match mongo.db.create_collection("test_users", None).await {
            Ok(_) => mongo
                .insert_user(user.clone(), Some("test_users"))
                .await
                .is_ok(),
            Err(_) => false,
        };
        teardown(mongo).await;
        assert!(res);
    }

    #[serial]
    #[tokio::test]
    async fn test_get_user() {
        let mongo = Mongo::init(DB_NAME).await;
        let res = match mongo.db.create_collection(COL_NAME, None).await {
            Ok(_) => {
                let user = User {
                    _id: Uuid::new(),
                    cards: HashMap::new(),
                };
                mongo
                    .insert_user(user.clone(), Some(COL_NAME))
                    .await
                    .unwrap();

                match mongo.get_user(user._id, Some(COL_NAME)).await {
                    Ok(Some(v)) => {
                        info!("Inserted user{:?}", user);
                        info!("Received User: {:?}", v);
                        v == user
                    }
                    _ => false,
                }
            }
            Err(_) => false,
        };
        teardown(mongo).await;
        assert!(res);
    }

    #[serial]
    #[tokio::test]
    async fn test_insert_merchant() {
        let mongo = Mongo::init(DB_NAME).await;
        let merchant = Merchant {
            _id: ObjectId::new(),
            name: "test".to_string(),
            address: "".to_string(),
            logo: "".to_string(),
        };
        let res = match mongo.db.create_collection("merchants", None).await {
            Ok(_) => match mongo.insert_merchant(merchant.clone()).await {
                Ok(_) => match mongo.get_merchant(merchant._id).await {
                    Some(v) => v == merchant,
                    None => false,
                },
                Err(_) => false,
            },
            Err(_) => false,
        };
        teardown(mongo).await;
        assert!(res);
    }
}
