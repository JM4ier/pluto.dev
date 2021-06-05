use super::schema::*;
use super::*;
use chrono::NaiveDateTime;
use org::*;

#[derive(Queryable, Insertable, Debug, PartialEq, Eq)]
#[table_name = "posts"]
pub struct Post {
    pub url: String,
    pub title: String,
    pub version: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub content: String,
    pub published: Option<NaiveDateTime>,
}

pub fn now() -> NaiveDateTime {
    chrono::Local::now().naive_utc()
}

#[derive(Queryable, Insertable, Debug, PartialEq, Eq)]
#[table_name = "tags"]
pub struct Tag {
    pub tag: String,
    pub url: String,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, PartialEq, Eq)]
#[table_name = "tags_meta"]
pub struct TagMeta {
    pub tag: String,
    pub display: bool,
    pub description: String,
}

impl Post {
    pub fn load_from_db(name: &str, db: &PgConnection) -> AResult<Self> {
        use crate::schema::posts::dsl::*;
        if let Some(post) = posts
            .filter(url.eq(name))
            .load::<models::Post>(db)?
            .into_iter()
            .next()
        {
            Ok(post)
        } else {
            Err(format!("no post with name `{}`.", name))?
        }
    }
}

impl Linkable for Post {
    fn link(&self) -> String {
        format!(
            r#"<a href="{}">{}</a>"#,
            PageKind::Post.url_of(&self.url),
            self.title
        )
    }
}

impl Linkable for Tag {
    fn link(&self) -> String {
        format!(
            r#"<a href="{}">{}</a> "#,
            PageKind::Tag.url_of(&self.tag),
            self.tag.to_uppercase(),
        )
    }
}
