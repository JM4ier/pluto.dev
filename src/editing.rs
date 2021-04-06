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
                published: post.published,
                tags,
            },
            content: post.content,
        })
    }

    pub fn write_to_db(self, name: &str, db: &PgConnection) -> AResult<()> {
        let orig_post = models::Post::load_from_db(name, db);

        let edited = models::Post {
            url: name.into(),
            created: orig_post.map(|p| p.created).unwrap_or(models::now()),
            updated: models::now(),
            title: self.meta.title,
            version: self.meta.version,
            published: self.meta.published,
            content: self.content,
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

fn open_editor(path: &str) -> AResult<()> {
    std::process::Command::new("/usr/bin/sh")
        .arg("-c")
        .arg(format!("vim {}", path))
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn edit(url: &str, db: &PgConnection) -> AResult<()> {
    const EDIT_PATH: &str = ".edit.md";

    db.build_transaction().run(|| {
        match Post::new_from_db(url, db) {
            Ok(post) => post.write_to_file(EDIT_PATH)?,
            Err(_) => std::fs::write(EDIT_PATH, vec![])?,
        }

        let edited = loop {
            open_editor(EDIT_PATH)?;
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
