use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PostMeta {
    pub title: String,
    #[serde(default)]
    pub version: String,
    pub published: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub struct Post {
    pub meta: PostMeta,
    pub content: String,
}

impl Post {
    pub fn new_from_file(path: &str) -> AResult<Self> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut parts = content.split("---");
        parts.next().ok_or("missing metadata")?;
        let meta = parts.next().ok_or("missing metadata")?;
        let markdown = parts.collect::<Vec<_>>().join("---");
        let meta = serde_yaml::from_str::<PostMeta>(meta)?;

        Ok(Self {
            meta,
            content: markdown,
        })
    }

    pub fn write_to_file(&self, path: &str) -> AResult<()> {
        let mut buffer = serde_yaml::to_string(&self.meta)?;
        buffer += &format!("---{}", self.content);
        std::fs::write(path, buffer)?;
        Ok(())
    }

    pub fn new_from_db(name: &str, db: &PgConnection) -> AResult<Self> {
        let post = models::Post::load_from_db(name, db)?;

        let tags = {
            use crate::schema::tags::dsl::*;
            tags.filter(url.eq(name)).select(tag).load::<String>(db)?
        };

        Ok(Self {
            meta: PostMeta {
                title: post.title,
                version: post.version,
                published: post.published.is_some(),
                tags,
            },
            content: post.content,
        })
    }

    pub fn write_to_db(self, name: &str, db: &PgConnection) -> AResult<()> {
        let orig_post = models::Post::load_from_db(name, db);

        let published = orig_post
            .as_ref()
            .ok()
            .map(|p| p.published)
            .flatten()
            .unwrap_or(models::now());
        let published = if self.meta.published {
            Some(published)
        } else {
            None
        };

        let edited = models::Post {
            url: name.into(),
            created: orig_post
                .as_ref()
                .map(|p| p.created)
                .unwrap_or(models::now()),
            updated: models::now(),
            title: self.meta.title,
            version: self.meta.version,
            content: self.content,
            published,
        };

        use diesel::dsl::*;

        {
            use crate::schema::posts::dsl::*;
            delete(posts.filter(url.eq(name))).execute(db)?;
            insert_into(schema::posts::table)
                .values(edited)
                .execute(db)?;
        }

        {
            use crate::schema::tags::dsl::*;

            let tag_tuples = self
                .meta
                .tags
                .into_iter()
                .map(|t| models::Tag {
                    tag: t,
                    url: name.into(),
                })
                .collect::<Vec<_>>();

            delete(tags.filter(url.eq(name))).execute(db)?;
            insert_into(tags).values(&tag_tuples).execute(db)?;
        }

        Ok(())
    }
}

const EDIT_PATH: &str = ".edit.md";
fn open_editor() -> AResult<()> {
    std::process::Command::new("/usr/bin/sh")
        .arg("-c")
        .arg(format!("vim {}", EDIT_PATH))
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn edit_post(url: &str, db: &PgConnection) -> AResult<()> {
    db.build_transaction().run(|| {
        match Post::new_from_db(url, db) {
            Ok(post) => post.write_to_file(EDIT_PATH)?,
            Err(_) => std::fs::write(EDIT_PATH, vec![])?,
        }

        let edited = loop {
            open_editor()?;
            match Post::new_from_file(EDIT_PATH) {
                Ok(post) => break post,
                Err(err) => {
                    eprintln!("{}", err);
                    eprintln!("Press q to exit.");

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if input.starts_with("q") {
                        return Err(err);
                    }
                }
            }
        };

        edited.write_to_db(url, db)?;

        Ok(())
    })
}

pub fn edit_tag(name: &str, db: &PgConnection) -> AResult<()> {
    use crate::models::*;
    use crate::schema::tags_meta::dsl::*;
    use diesel::dsl::*;

    db.build_transaction().run(|| {
        let meta = tags_meta.filter(tag.eq(name)).load::<TagMeta>(db)?;
        let mut meta = meta.into_iter().next().unwrap_or(TagMeta {
            tag: name.into(),
            display: true,
            description: String::from(""),
        });

        std::fs::write(EDIT_PATH, meta.description.as_bytes())?;
        open_editor()?;
        meta.description = std::fs::read_to_string(EDIT_PATH)?;

        insert_into(tags_meta)
            .values(&meta)
            .on_conflict(tag)
            .do_update()
            .set(&meta)
            .execute(db)?;

        Ok(())
    })
}
