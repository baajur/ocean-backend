use super::*;
use crate::model::mandela;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

// mandela.create
pub fn create(data: RequestData) -> RequestResult {
    #[derive(Deserialize)]
    struct Req {
        title_mode: i32,
        title: String,
        what: String,
        before: String,
        after: String,
        description: String,
        images: serde_json::Value,
        videos: serde_json::Value,
        links: serde_json::Value,
        user_id: i32,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    use crate::model::schema::mandels::dsl::*;

    let new_mandela = mandela::NewMandela {
        title_mode: req.title_mode,
        title: req.title,
        what: req.what,
        before: req.before,
        after: req.after,
        description: req.description,
        images: req.images,
        videos: req.videos,
        links: req.links,
        user_id: req.user_id,
    };
    let mandela_id = diesel::insert_into(mandels)
        .values(&new_mandela)
        .returning(id)
        .get_result::<i32>(&data.db.conn)?;

    let result = json!({ "id": mandela_id });

    Ok(Some(result))
}

// mandela.update
pub fn update(data: RequestData) -> RequestResult {
    use crate::model::schema::mandels;
    use crate::model::schema::mandels::dsl::*;
    #[derive(Deserialize)]
    struct Req {
        id: i32,
        title_mode: i32,
        title: String,
        what: String,
        before: String,
        after: String,
        description: String,
        images: serde_json::Value,
        videos: serde_json::Value,
        links: serde_json::Value,
        user_id: i32,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    let update_mandela = mandela::UpdateMandela {
        title_mode: req.title_mode,
        title: req.title,
        what: req.what,
        before: req.before,
        after: req.after,
        description: req.description,
        images: req.images,
        videos: req.videos,
        links: req.links,
        user_id: req.user_id,
    };

    diesel::update(mandels.filter(mandels::id.eq(req.id)))
        .set(&update_mandela)
        .execute(&data.db.conn)?;

    Ok(None)
}

// mandela.getOne
pub fn get_one(data: RequestData) -> RequestResult {
    use crate::model::schema::mandels;
    use crate::model::schema::mandels::dsl::*;
    use crate::model::schema::marks;
    use crate::model::schema::marks::dsl::*;

    #[derive(Deserialize)]
    struct Req {
        id: i32,
        user_id: Option<i32>,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;
    let mark_user_id = if let Some(i) = req.user_id { i } else { 0 };

    #[derive(Queryable, Serialize)]
    pub struct Mandela {
        id: i32,
        title: String,
        title_mode: i32,
        description: String,
        user_id: i32,
        images: serde_json::Value,
        videos: serde_json::Value,
        links: serde_json::Value,
        create_ts: NaiveDateTime,
        update_ts: NaiveDateTime,
        what: String,
        before: String,
        after: String,
        mark_ts: Option<NaiveDateTime>,
    }

    let record = mandels
        .left_join(
            marks.on(marks::user_id
                .eq(mark_user_id)
                .and(marks::mandela_id.eq(mandels::id))),
        )
        .select((
            mandels::id,
            title,
            title_mode,
            description,
            mandels::user_id,
            images,
            videos,
            links,
            mandels::create_ts,
            update_ts,
            what,
            before,
            after,
            marks::create_ts.nullable(),
        ))
        .filter(mandels::id.eq(req.id))
        .limit(1)
        .load::<Mandela>(&data.db.conn)?;

    let result = serde_json::to_value(&record)?;
    Ok(Some(result))
}

// mandela.getAll
pub fn get_all(data: RequestData) -> RequestResult {
    use crate::model::schema::comments;
    use crate::model::schema::comments::dsl::*;
    use crate::model::schema::mandels;
    use crate::model::schema::mandels::dsl::*;
    use crate::model::schema::marks;
    use crate::model::schema::marks::dsl::*;
    use crate::model::schema::users;
    use crate::model::schema::users::dsl::*;
    use diesel::dsl::*;

    #[derive(Deserialize)]
    struct Req {
        offset: i64,
        limit: i64,
        user_id: Option<i32>,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    #[derive(Queryable, Serialize)]
    struct MandelaResp {
        id: i32,
        title_mode: i32,
        title: String,
        what: String,
        before: String,
        after: String,
        create_ts: NaiveDateTime,
        user_name: Option<String>,
        user_id: i32,
        comment_count: i32,
        mark_ts: Option<NaiveDateTime>,
    }

    let mark_user_id = if let Some(i) = req.user_id { i } else { 0 };

    let mut list = mandels
        .inner_join(users)
        .left_join(
            marks.on(marks::user_id
                .eq(mark_user_id)
                .and(marks::mandela_id.eq(mandels::id))),
        )
        .select((
            mandels::id,
            title_mode,
            title,
            what,
            before,
            after,
            mandels::create_ts,
            users::name,
            users::id,
            mandels::id, // Hack to fill by anything the last value
            marks::create_ts.nullable(),
        ))
        .order(mandels::id.desc())
        .offset(req.offset)
        .limit(req.limit)
        .load::<MandelaResp>(&data.db.conn)?;

    for elem in &mut list {
        let comment_count: i64 = comments
            .filter(comments::mandela_id.eq(elem.id))
            .select(count_star())
            .first(&data.db.conn)?;
        elem.comment_count = comment_count as i32;
    }

    let total_count: i64 = mandels.select(count_star()).first(&data.db.conn)?;

    #[derive(Serialize)]
    struct Resp {
        total_count: i64,
        mandels: Vec<MandelaResp>,
    };

    let resp = serde_json::to_value(&Resp {
        total_count: total_count,
        mandels: list,
    })?;

    let result = serde_json::to_value(&resp)?;
    Ok(Some(result))
}

// mandela.delete
pub fn delete(data: RequestData) -> RequestResult {
    #[derive(Deserialize)]
    struct Req {
        id: Vec<i32>,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    use crate::model::schema::mandels::dsl::*;

    diesel::delete(mandels.filter(id.eq_any(req.id))).execute(&data.db.conn)?;
    Ok(None)
}

// mandela.mark
pub fn mark(data: RequestData) -> RequestResult {
    #[derive(Deserialize)]
    struct Req {
        id: i32,
        user_id: i32,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    use crate::model::schema::marks;
    use crate::model::schema::marks::dsl::*;

    #[derive(Insertable)]
    #[table_name = "marks"]
    pub struct NewMark {
        mandela_id: i32,
        user_id: i32,
    };

    let new_mark = NewMark {
        mandela_id: req.id,
        user_id: req.user_id,
    };

    diesel::insert_into(marks)
        .values(&new_mark)
        .execute(&data.db.conn)?;
    Ok(None)
}
