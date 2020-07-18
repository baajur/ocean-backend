use super::*;
use crate::api;
use crate::model::user;
use crate::model::user_group;
use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;

// user.create
pub fn create(data: RequestData) -> RequestResult {
    use crate::model::schema::user_groups::dsl::*;
    use crate::model::schema::users;
    use crate::model::schema::users::dsl::*;
    #[derive(Deserialize)]
    struct Req {
        name: Option<String>,
        code: String,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    let groups = user_groups
        .filter(code.eq(req.code))
        .limit(1)
        .load::<user_group::UserGroup>(&data.db.conn)?;

    let new_user = user::NewUser {
        name: req.name,
        group_id: groups[0].id,
    };

    let user_id = diesel::insert_into(users)
        .values(&new_user)
        .returning(users::id)
        .get_result::<i32>(&data.db.conn)?;

    let result = json!({
        "id": user_id,
    });

    Ok(Some(result))
}

// user.auth
pub fn auth(data: RequestData) -> RequestResult {
    use crate::model::schema::user_groups;
    use crate::model::schema::user_groups::dsl::*;
    use crate::model::schema::users;
    use crate::model::schema::users::dsl::*;

    #[derive(Deserialize)]
    struct Req {
        id: i32,
        password: String,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    let result = users
        .filter(users::id.eq(req.id))
        .load::<user::User>(&data.db.conn)?;

    let request_token = sha1_token(req.id, req.password);

    if result.is_empty() || result[0].token != request_token {
        Err(api::make_error(api::error::WRONG_USER_PASSWORD))
    } else {
        let user_group = user_groups
            .filter(user_groups::id.eq(result[0].group_id))
            .limit(1)
            .load::<user_group::UserGroup>(&data.db.conn)?;

        Ok(Some(json!({ "token": request_token,
             "code": user_group[0].code,
             "name": result[0].name })))
    }
}

fn sha1_token(id: i32, password: String) -> String {
    let mut sha = sha1::Sha1::new();
    sha.update((id.to_string() + &password).as_bytes());
    sha.digest().to_string()
}

// user.getOne
pub fn get_one(data: RequestData) -> RequestResult {
    use crate::model::schema::user_groups;
    use crate::model::schema::user_groups::dsl::*;
    use crate::model::schema::users::dsl::*;

    let params = data.params.unwrap();
    let user_token = params["token"].as_str().unwrap();

    let user = users
        .filter(token.eq(user_token))
        .limit(1)
        .load::<user::User>(&data.db.conn)?;

    let user_group = user_groups
        .filter(user_groups::id.eq(user[0].group_id))
        .limit(1)
        .load::<user_group::UserGroup>(&data.db.conn)?;

    let result = json!({
        "id": user[0].id,
        "name": user[0].name,
        "code": user_group[0].code,
        "create_ts": user[0].create_ts
    });

    Ok(Some(result))
}

// user.update
pub fn update(data: RequestData) -> RequestResult {
    use crate::model::schema::user_groups::dsl::*;
    use crate::model::schema::users;
    use crate::model::schema::users::dsl::*;

    #[derive(Deserialize)]
    struct Req {
        id: i32,
        name: String,
        code: String,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    let groups = user_groups
        .filter(code.eq(req.code))
        .limit(1)
        .load::<user_group::UserGroup>(&data.db.conn)?;

    #[derive(AsChangeset)]
    #[table_name = "users"]
    pub struct UpdateUser {
        pub name: String,
        pub group_id: i32,
        pub update_ts: NaiveDateTime,
    }

    let update_user = UpdateUser {
        name: req.name,
        group_id: groups[0].id,
        update_ts: Utc::now().naive_utc(),
    };

    diesel::update(users.filter(users::id.eq(req.id)))
        .set(&update_user)
        .execute(&data.db.conn)?;

    Ok(None)
}

// user.changePassword
pub fn change_password(data: RequestData) -> RequestResult {
    use crate::model::schema::users::dsl::*;

    #[derive(Deserialize)]
    struct Req {
        id: i32,
        password: String,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;
    let user_token = sha1_token(req.id, req.password);

    diesel::update(users.filter(id.eq(req.id)))
        .set(token.eq(user_token.clone()))
        .execute(&data.db.conn)?;

    let result = json!({ "token": user_token });

    Ok(Some(result))
}
