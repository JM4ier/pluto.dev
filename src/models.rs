use super::schema::posts;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Queryable, Insertable, Deserialize, Debug, PartialEq, Eq)]
#[table_name = "posts"]
pub struct Post {
    #[serde(skip, default)]
    pub path: String,
    pub title: String,
    #[serde(default)]
    pub version: String,
    pub published: bool,
    #[serde(skip, default = "now")]
    pub created: NaiveDateTime,
    #[serde(skip, default)]
    pub updated: Option<NaiveDateTime>,
}

fn now() -> NaiveDateTime {
    chrono::Local::now().naive_utc()
}

impl Post {
    pub fn file_name<'s>(&'s self) -> &'s str {
        use std::path::Path;
        Path::new(&self.path)
            .file_stem()
            .expect("invalid file name")
            .to_str()
            .unwrap()
    }
    pub fn out_path(&self) -> String {
        format!("html/{}", self.file_name())
    }
    pub fn relative_link_to(&self, other: &Self) -> String {
        format!("./{}", other.file_name())
    }
}
