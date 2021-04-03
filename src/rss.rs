use super::*;
use crate::schema::posts::dsl::{created, posts, published};
use quick_xml::se::to_string;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct BString(String);

impl<'s> From<&'s str> for BString {
    fn from(string: &'s str) -> Self {
        Self(string.into())
    }
}

impl From<String> for BString {
    fn from(string: String) -> Self {
        Self(string)
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(rename = "channel")]
struct Channel {
    title: BString,
    link: BString,
    description: BString,
    #[serde(rename = "item")]
    items: Vec<Item>,
}

#[derive(Serialize, Clone)]
struct Item {
    title: BString,
    link: BString,
    guid: BString,
    #[serde(rename = "pubDate")]
    pub_date: BString,
    description: BString,
}

pub fn create_feed(db: &PgConnection) -> AResult<String> {
    let url = "https://pluto.dev/";
    let items = posts
        .filter(published)
        .order_by(created.desc())
        .limit(20)
        .load::<Post>(db)?
        .into_iter()
        .map(|item| {
            let link = BString::from(format!("{}{}", url, item.url));
            let pub_date = format!("{}", item.created.format("%a, %d %b %Y %H:%M:%S")).into();
            let description = crate::render::render_raw(&item.content).into();
            Item {
                title: item.title.into(),
                guid: link.clone(),
                link,
                pub_date,
                description,
            }
        })
        .collect();

    let channel = Channel {
        title: "Jonas' personal website".into(),
        link: url.into(),
        description: "Here I'll post stuff from time to time.".into(),
        items,
    };

    let xml = to_string(&channel)?;

    let rss = format!(
        r#"<?xml version="1.0" encoding="UTF-8" ?><rss version="2.0">{}</rss> "#,
        xml
    );

    Ok(rss)
}
