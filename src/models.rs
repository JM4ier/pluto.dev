use super::schema::posts;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[table_name = "posts"]
pub struct Post {
    #[serde(skip, default)]
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub version: String,
    pub published: bool,
    #[serde(skip, default = "now")]
    pub created: NaiveDateTime,
    #[serde(skip, default = "now")]
    pub updated: NaiveDateTime,
    #[serde(skip, default)]
    pub content: String,
}

impl Post {}

fn now() -> NaiveDateTime {
    chrono::Local::now().naive_utc()
}
