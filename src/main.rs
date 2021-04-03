use pulldown_cmark::*;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use walkdir::WalkDir;

mod code;
mod models;
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

fn files(path: &str) -> Vec<String> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.metadata().map_or(false, |m| m.is_file()))
        .map(|e| e.path().to_string_lossy().into())
        .collect()
}

fn to_html(meta: &Post, db: &PgConnection) -> AResult<String> {
    let markdown = &meta.content;
    let parser = Parser::new(markdown);

    let mut code_lang = None;

    use Event::*;
    let parser = parser.flat_map(|event| match &event {
        Start(Tag::CodeBlock(kind)) => {
            if let CodeBlockKind::Fenced(kind) = kind {
                code_lang = Some(kind.clone().into_string());
            }
            vec![Html(r#"<div class="code">"#.into()), event]
        }
        Text(code) => code_lang
            .as_ref()
            .map(|lang| code::highlight(&code, &lang).ok())
            .flatten()
            .map_or(vec![event], |html| vec![Html(html.into())]),
        End(Tag::CodeBlock(_)) => {
            code_lang = None;
            vec![event, Html("</div>".into())]
        }
        _ => {
            vec![event]
        }
    });

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    Ok(format!(
        include_str!("skeleton.html"),
        body = html_out,
        title = meta.title,
        copyright = copyright_years(&meta.created, &meta.updated),
        bottom_navigation = bottom_navigation(meta, db)?,
    ))
}

use chrono::NaiveDateTime;
fn copyright_years(from: &NaiveDateTime, to: &NaiveDateTime) -> String {
    let mut copyright = format!("{}", from.date().format("%Y"));
    let to = format!("{}", to.date().format("%Y"));
    if copyright != to {
        copyright = format!("{}-{}", copyright, to);
    }
    copyright
}

fn bottom_navigation(this: &Post, db: &PgConnection) -> AResult<String> {
    use crate::schema::posts::dsl::*;

    let prev = posts
        .filter(created.lt(&this.created).and(published))
        .order_by(created.desc())
        .limit(1)
        .load::<Post>(db)?;

    let next = posts
        .filter(created.gt(&this.created).and(published))
        .order_by(created.asc())
        .limit(1)
        .load::<Post>(db)?;

    let first = posts
        .filter(published)
        .order_by(created.asc())
        .limit(1)
        .load::<Post>(db)?;

    let last = posts
        .filter(published)
        .order_by(created.desc())
        .limit(1)
        .load::<Post>(db)?;

    let link = |dir, linked: &Post| {
        format!(
            r#" <a href="{}" class="bottom-nav-button">{}</a> "#,
            linked.url, dir
        )
    };

    let (lname, llink) = match prev.first() {
        Some(first) => ("Prev", first),
        None => ("Last", last.first().unwrap()),
    };

    let (rname, rlink) = match next.first() {
        Some(first) => ("Next", first),
        None => ("First", first.first().unwrap()),
    };

    let mut links = String::new();

    links += &link(format!("← {}", lname), llink);
    links += &link(format!("{} →", rname), rlink);

    Ok(links)
}

fn render_overview(db: &PgConnection) -> AResult<()> {
    let mut body = String::from("<h1>Blog Posts</h1>");
    body += "<hr>";

    use crate::schema::posts::dsl::*;
    let sites = posts
        .filter(published)
        .order_by(created.desc())
        .load::<Post>(db)?;

    body += r#"<table class="post-list">"#;
    body += "<th>Post</th><th>Date</th>";
    for site in sites.iter() {
        body += &format!(
            r#"<tr><td><a href="{}">{}</a></td><td>{}</td></tr>"#,
            site.url,
            site.title,
            site.created.date().format("%d-%m-%Y")
        );
    }
    body += "</table>";

    let page = format!(
        include_str!("skeleton.html"),
        title = "Overview",
        body = body,
        bottom_navigation = "",
        copyright = copyright_years(
            &sites.last().unwrap().created,
            &sites.first().unwrap().created
        )
    );

    std::fs::write("html/index.html", page)?;

    Ok(())
}

fn process(file: &str, db: &PgConnection, force_render: bool) -> AResult<()> {
    use crate::schema::posts::dsl::*;
    use diesel::dsl::*;

    let mut meta = read_md(file)?;
    meta.url = file.into();

    let existing = posts.filter(url.eq(&meta.url)).limit(1).load::<Post>(db)?;

    if let Some(entry) = existing.first() {
        meta.updated = meta.created;
        meta.created = entry.created;

        if meta.version == entry.version {
            if force_render && meta.published {
                //render(&meta, db)?;
            }
            return Ok(());
        }
    }

    delete(posts.filter(url.eq(&meta.url))).execute(db)?;
    insert_into(schema::posts::table)
        .values(&meta)
        .execute(db)?;

    println!("inserting or updating `{}`.", meta.url);

    if meta.published {
        //render(&meta, db)?;
    }

    Ok(())
}

fn read_md(path: &str) -> AResult<Post> {
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

fn write_md(path: &str, post: &Post) -> AResult<()> {
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
        write_md(edit_path, entry)?;
    } else {
        std::fs::write(edit_path, "")?;
    }

    let mut edited = loop {
        open_editor(edit_path)?;
        match read_md(edit_path) {
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
    let entries = posts.filter(url.like(filter)).limit(100).load::<Post>(db)?;

    println!("{:>24}: {}", "URL", "TITLE");
    for entry in entries.iter() {
        println!("{:>24}: {}", entry.url, entry.title);
    }

    Ok(())
}

fn render_all(db: &PgConnection) -> AResult<()> {
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
    } else if let Some(filter) = matches.value_of("list") {
        list(filter, &connection)?;
    } else if matches.is_present("render") {
        render_all(&connection)?;
    }
    Ok(())
}
