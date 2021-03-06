use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

mod code;
mod config;
mod editing;
mod models;
mod org;
mod polyring;
mod render;
mod rss;
mod schema;

use models::Post;
use org::*;

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use dotenv::dotenv;
use std::env;

type AResult<T> = Result<T, Box<dyn Error>>;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set.");
    PgConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url))
}

fn list(filter: &str, db: &PgConnection) -> AResult<()> {
    use crate::schema::posts::dsl::*;

    let filter = format!("%{}%", filter);
    let entries = posts
        .filter(url.like(filter))
        .order_by(created.desc())
        .limit(100)
        .load::<Post>(db)?;

    println!("{:^25} | {:^10} | {:^50}", "URL", "PUBLISHED", "TITLE");
    println!("{:-^25}-+-{:-^10}-+-{:-^50}", "", "", "");
    for entry in entries.iter() {
        let publ = entry.published.map(|_| "yes").unwrap_or("no");
        println!("{:^25} | {:^10} | {:^50}", entry.url, publ, entry.title);
    }

    Ok(())
}

pub struct RenderConfig {
    preview: bool,
}

fn render_all(db: &PgConnection, config: &RenderConfig) -> AResult<()> {
    use crate::schema::posts::dsl::*;
    use fs_extra::dir::{copy, CopyOptions};

    std::fs::remove_dir_all("html").ok();
    std::fs::create_dir("html")?;

    for kind in PageKind::kinds() {
        std::fs::create_dir(kind.dir())?;
    }

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.content_only = true;
    copy("static_html", "html", &options)?;

    println!("rendering rss.");
    let rss = rss::create_feed(db)?;
    std::fs::write("html/rss.xml", rss)?;

    let pages = if config.preview {
        posts.load::<Post>(db)?
    } else {
        posts.filter(published.is_not_null()).load::<Post>(db)?
    };

    for page in pages.iter() {
        println!("rendering page {}.", page.url);
        let rendered = render::blogpost(page, db)?;
        std::fs::write(PageKind::Post.path_of(&page.url), rendered)?;
    }

    let tags = {
        use schema::tags_meta::dsl::*;
        tags_meta.select(tag).load::<String>(db)?
    };
    for tag in tags.iter() {
        println!("rendering tag {}.", tag);
        let rendered = render::tag(&tag, db)?;
        std::fs::write(PageKind::Tag.path_of(&tag), rendered)?;
    }

    println!("rendering overview.");
    let overview = render::overview(db, config)?;
    std::fs::write("html/index.html", overview)?;

    Ok(())
}

fn transfer() -> AResult<()> {
    use config::CONFIG;
    use std::process::Command;

    println!("Removing old files");
    Command::new("/usr/bin/ssh")
        .arg(&CONFIG.ssh_url)
        .arg("rm")
        .arg("-rf")
        .arg("html")
        .spawn()?
        .wait()?;

    println!("Transferring new files");
    Command::new("/usr/bin/scp")
        .arg("-r")
        .arg("html")
        .arg(format!("{}:~", CONFIG.ssh_url))
        .spawn()?
        .wait()?;

    Ok(())
}

fn main() -> AResult<()> {
    use rustop::opts;
    let (args, _) = opts! {
        synopsis "This is a tool to manage the website hosted on pluto.dev.";
        opt edit: Option<String>,   desc: "Edit a post";
        opt list: Option<String>,   desc: "Display a list of recent posts.";
        opt render: bool,           desc: "Renders the website.";
        opt tag: Option<String>,    desc: "Edit the description of a tag.";
        opt send: bool,             desc: "Transfers the files to the server.";
        opt preview: bool,          desc: "Preview rendering: also renders unpublished posts";
    }
    .parse_or_exit();

    let connection = establish_connection();

    if let Some(post) = args.edit {
        editing::edit_post(&post, &connection)?;
    }
    if let Some(tag) = args.tag {
        editing::edit_tag(&tag, &connection)?;
    }
    if let Some(filter) = args.list {
        list(&filter, &connection)?;
    }
    if args.render {
        let config = RenderConfig {
            preview: args.preview,
        };
        render_all(&connection, &config)?;
    }
    if args.send && !args.preview {
        transfer()?;
    }
    Ok(())
}
