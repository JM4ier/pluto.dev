use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

mod code;
mod config;
mod models;
mod polyring;
mod render;
mod rss;
mod schema;

use models::Post;

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

fn read_post_from_file(path: &str) -> AResult<Post> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut parts = content.split("---");
    parts.next().ok_or("missing metadata")?;
    let meta = parts.next().ok_or("missing metadata")?;
    let markdown = parts.collect::<Vec<_>>().join("---");

    let mut meta = serde_yaml::from_str::<Post>(meta)?;
    meta.content = markdown;
    Ok(meta)
}

fn write_post_to_file(path: &str, post: &Post) -> AResult<()> {
    let mut buffer = serde_yaml::to_string(post)?;
    buffer += &format!("---{}", post.content);
    std::fs::write(path, buffer)?;
    Ok(())
}

fn open_editor(path: &str) -> AResult<()> {
    std::process::Command::new("/usr/bin/sh")
        .arg("-c")
        .arg(format!("vim {}", path))
        .spawn()?
        .wait()?;
    Ok(())
}

fn edit(post: &str, db: &PgConnection) -> AResult<()> {
    use crate::schema::posts::dsl::*;
    use diesel::dsl::*;

    let entry = posts.filter(url.eq(post)).load::<Post>(db)?;

    let edit_path = ".edit.md";

    if let Some(entry) = entry.first() {
        write_post_to_file(edit_path, entry)?;
    } else {
        std::fs::write(edit_path, "")?;
    }

    let mut edited = loop {
        open_editor(edit_path)?;
        match read_post_from_file(edit_path) {
            Ok(post) => break post,
            Err(err) => {
                eprintln!("{}", err);
                eprintln!("Press enter to fix the file.");
                std::io::stdin().read_line(&mut String::new()).ok();
            }
        }
    };

    edited.url = post.into();

    if let Some(entry) = entry.first() {
        // keep original creation date
        edited.created = entry.created;
    }

    delete(posts.filter(url.eq(post))).execute(db)?;
    insert_into(schema::posts::table)
        .values(edited)
        .execute(db)?;

    Ok(())
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
        let publ = if entry.published { "yes" } else { "no" };
        println!("{:^25} | {:^10} | {:^50}", entry.url, publ, entry.title);
    }

    Ok(())
}

fn render_all(db: &PgConnection) -> AResult<()> {
    use crate::schema::posts::dsl::*;
    use fs_extra::dir::{copy, CopyOptions};

    println!("{:#?}", *polyring::MEMBERS);

    std::fs::remove_dir_all("html").ok();
    std::fs::create_dir("html")?;
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.content_only = true;
    copy("static_html", "html", &options)?;

    let rss = rss::create_feed(db)?;
    std::fs::write("html/rss.xml", rss)?;

    let pages = posts.filter(published).load::<Post>(db)?;
    for page in pages.iter() {
        println!("rendering {}.", page.url);
        let rendered = render::blogpost(page, db)?;
        std::fs::write(format!("html/{}", page.url), rendered)?;
    }

    println!("rendering overview.");
    let overview = render::overview(db)?;
    std::fs::write("html/index.html", overview)?;

    Ok(())
}

use clap::{App, Arg};
fn main() -> AResult<()> {
    let matches = App::new("Website Manager")
        .arg(
            Arg::with_name("edit")
                .short("e")
                .long("edit")
                .value_name("POST")
                .help("Edits a post.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Displays posts.")
                .value_name("FILTER")
                .default_value("")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("render")
                .short("r")
                .long("render")
                .help("Renders all posts.")
                .takes_value(false),
        )
        .get_matches();

    let connection = establish_connection();

    if let Some(post) = matches.value_of("edit") {
        edit(post, &connection)?;
    } else if matches.is_present("render") {
        render_all(&connection)?;
    } else if let Some(filter) = matches.value_of("list") {
        list(filter, &connection)?;
    }
    Ok(())
}
