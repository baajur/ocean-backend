use crate::model::date_serializer;
use crate::model::schema::topics;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct Topic {
    pub id: i32,
    pub title: String,
    pub description: String,
    #[serde(with = "date_serializer")]
    pub create_ts: NaiveDateTime,
    #[serde(with = "date_serializer")]
    pub update_ts: NaiveDateTime,
    pub user_id: i32,
}

#[derive(Insertable)]
#[table_name = "topics"]
pub struct NewTopic<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub user_id: i32,
}
