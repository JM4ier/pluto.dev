use super::*;
use pulldown_cmark::*;

pub fn render_raw(post: &str) -> String {
    let parser = Parser::new(post);
    let mut html = String::new();
    html::push_html(&mut html, parser);
    html
}

fn render_markdown(post: &str) -> String {
    let parser = Parser::new(post);

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

    html_out
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

pub fn blogpost(post: &Post, db: &PgConnection) -> AResult<String> {
    let html = render_markdown(&post.content);
    Ok(format!(
        include_str!("skeleton.html"),
        body = html,
        title = post.title,
        copyright = copyright_years(&post.created, &post.updated),
        bottom_navigation = bottom_navigation(post, db)?,
    ))
}

pub fn overview(db: &PgConnection) -> AResult<String> {
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
    body += "<hr>";
    body += &crate::polyring::BANNER;

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

    Ok(page)
}
