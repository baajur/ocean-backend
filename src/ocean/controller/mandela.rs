use super::*;
use crate::model::mandela;
use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::Int2;
use diesel::sql_types::Int8;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

#[derive(QueryableByName, Serialize)]
struct Votes {
    #[sql_type = "Int2"]
    vote: i16,
    #[sql_type = "Int8"]
    count: i64,
}

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
        categories: serde_json::Value,
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

    use crate::model::schema::categories;

    let category_numbers: Vec<i16> = serde_json::from_value(req.categories).unwrap();

    #[derive(Insertable, Deserialize)]
    #[table_name = "categories"]
    pub struct NewCategoryNumber {
        mandela_id: i32,
        number: i16,
    }

    for number in category_numbers.into_iter() {
        let new_category_number = NewCategoryNumber {
            mandela_id: mandela_id,
            number: number,
        };

        diesel::insert_into(categories::table)
            .values(&new_category_number)
            .execute(&data.db.conn)?;
    }

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
        update_ts: Utc::now().naive_utc(),
    };

    diesel::update(mandels.filter(mandels::id.eq(req.id)))
        .set(&update_mandela)
        .execute(&data.db.conn)?;

    Ok(None)
}

fn get_poll(db: &db::Db, mandela_id: i32) -> Vec<Votes> {
    use diesel::dsl::*;
    use diesel::sql_types::Int4;

    sql_query(
        "SELECT vote, COUNT (*) as count FROM votes AS v
    JOIN mandels AS m ON m.id = v.mandela_id
    WHERE m.id = $1
    GROUP BY vote",
    )
    .bind::<Int4, _>(mandela_id)
    .load::<Votes>(&db.conn)
    .unwrap()
}

// mandela.getOne
pub fn get_one(data: RequestData) -> RequestResult {
    use crate::model::schema::mandels;
    use crate::model::schema::mandels::dsl::*;
    use crate::model::schema::marks;
    use crate::model::schema::marks::dsl::*;
    use crate::model::schema::users;
    use crate::model::schema::users::dsl::*;

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
        user_name: Option<String>,
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

    let mandela_record = mandels
        .inner_join(users)
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
            users::name,
            images,
            videos,
            links,
            mandels::create_ts,
            mandels::update_ts,
            what,
            before,
            after,
            marks::create_ts.nullable(),
        ))
        .filter(mandels::id.eq(req.id))
        .first::<Mandela>(&data.db.conn)?;

    use crate::model::schema::votes;
    use crate::model::schema::votes::dsl::*;
    use diesel::dsl::*;

    let mandela_votes: Option<Vec<Votes>> = if let Some(i) = req.user_id {
        let vote_exist = select(exists(
            votes.filter(votes::mandela_id.eq(req.id).and(votes::user_id.eq(i))),
        ))
        .get_result::<bool>(&data.db.conn)
        .unwrap();

        let v: Option<Vec<Votes>> = if vote_exist {
            let votes_count = get_poll(&data.db, req.id);
            Some(votes_count)
        } else {
            None
        };

        v
    } else {
        None
    };

    #[derive(Serialize)]
    struct MandelaResp {
        mandela: Mandela,
        votes: Option<Vec<Votes>>,
    }

    let resp = MandelaResp {
        mandela: mandela_record,
        votes: mandela_votes,
    };

    let result = serde_json::to_value(&resp)?;
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
        filter: Option<i8>,
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

    let req_user_id = if let Some(i) = req.user_id { i } else { 0 };
    const SHOW_ALL: i8 = 0;
    const SHOW_NEW: i8 = 1;
    const SHOW_MINE: i8 = 2;

    let filter = if let Some(i) = req.filter {
        i
    } else {
        SHOW_ALL
    };

    let query = mandels
        .inner_join(users)
        .left_join(
            marks.on(marks::user_id
                .eq(req_user_id)
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
        ));

    let mut list = if filter == SHOW_ALL {
        query
            .order(mandels::id.desc())
            .offset(req.offset)
            .limit(req.limit)
            .load::<MandelaResp>(&data.db.conn)?
    } else if filter == SHOW_NEW {
        query
            .filter(marks::create_ts.is_null())
            .order(mandels::id.desc())
            .offset(req.offset)
            .limit(req.limit)
            .load::<MandelaResp>(&data.db.conn)?
    } else if filter == SHOW_MINE {
        query
            .filter(mandels::user_id.eq(req_user_id))
            .order(mandels::id.desc())
            .offset(req.offset)
            .limit(req.limit)
            .load::<MandelaResp>(&data.db.conn)?
    } else {
        panic!("Unknown query filter");
    };

    for elem in &mut list {
        let comment_count: i64 = comments
            .filter(comments::mandela_id.eq(elem.id))
            .select(count_star())
            .first(&data.db.conn)?;
        elem.comment_count = comment_count as i32;
    }

    let total_count: i64 = mandels.select(count_star()).first(&data.db.conn)?;
    let mut new_count = 0;
    let mut mine_count = 0;

    if let Some(i) = req.user_id {
        let mark_count: i64 = marks
            .select(count_star())
            .filter(marks::user_id.eq(i))
            .first(&data.db.conn)?;
        new_count = total_count - mark_count;
        mine_count = mandels
            .select(count_star())
            .filter(mandels::user_id.eq(i))
            .first(&data.db.conn)?;
    }

    #[derive(Serialize)]
    struct Resp {
        total_count: i64,
        new_count: i64,
        mine_count: i64,
        mandels: Vec<MandelaResp>,
    };

    let resp = serde_json::to_value(&Resp {
        total_count: total_count,
        new_count: new_count,
        mine_count: mine_count,
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

// mandela.vote
pub fn vote(data: RequestData) -> RequestResult {
    #[derive(Deserialize)]
    struct Req {
        id: i32,
        user_id: i32,
        vote: i16,
    }

    let req = serde_json::from_value::<Req>(data.params.unwrap())?;

    use crate::model::schema::votes;
    use crate::model::schema::votes::dsl::*;

    #[derive(Insertable)]
    #[table_name = "votes"]
    pub struct NewVote {
        mandela_id: i32,
        user_id: i32,
        vote: i16,
    };

    let new_vote = NewVote {
        mandela_id: req.id,
        user_id: req.user_id,
        vote: req.vote,
    };

    diesel::insert_into(votes)
        .values(&new_vote)
        .execute(&data.db.conn)?;

    let votes_count = get_poll(&data.db, req.id);
    let result = serde_json::to_value(&votes_count)?;
    Ok(Some(result))
}
