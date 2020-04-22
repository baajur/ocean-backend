use super::Controller;
use crate::db;
use crate::model::user;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json;
use serde_json::json;
use sha1;

pub struct User {}

#[derive(Deserialize)]
struct CreateRequest {
    name: Option<String>,
    password: String,
}

impl User {
    fn create(&self, db: &db::Db, params: Option<serde_json::Value>) -> Option<serde_json::Value> {
        let request: CreateRequest = serde_json::from_value(params.unwrap()).unwrap();

        use crate::model::schema::users::dsl::*;

        let new_user = user::NewUser {
            name: request.name,
            token: "dummy".to_string(),
        };

        let result: user::User = diesel::insert_into(users)
            .values(&new_user)
            // .returning(id)
            .get_result(&db.conn)
            .unwrap();

        let user_id = result.id;
        let user_token = &sha1_token(user_id, request.password);

        diesel::update(users.filter(id.eq(user_id)))
            .set(token.eq(user_token))
            .execute(&db.conn)
            .unwrap();

        let result = json!({
            "id": user_id,
            "token": user_token
        });

        Some(result)
    }
}

impl Controller for User {
    fn exec(
        &self,
        db: &db::Db,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Option<serde_json::Value> {
        match method {
            "create" => self.create(db, params),
            _ => {
                println!("method {} not found", method);
                None
            }
        }
    }
}

fn sha1_token(id: i32, password: String) -> String {
    let mut sha = sha1::Sha1::new();
    sha.update((id.to_string() + &password).as_bytes());
    sha.digest().to_string()
}
