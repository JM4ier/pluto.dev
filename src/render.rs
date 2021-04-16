use super::*;
use org::*;
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
            PageKind::Post.url_of(&linked.url),
            dir
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

fn tag_list(post_url: &str, db: &PgConnection) -> AResult<String> {
    use crate::schema::tags::dsl::*;
    let tag_urls = tags
        .filter(url.eq(post_url))
        .load::<models::Tag>(db)?
        .into_iter()
        .map(|t| t.link())
        .collect::<String>();
    if tag_urls.len() == 0 {
        Ok(String::new())
    } else {
        Ok(format!("<br><strong>Tags:</strong> {}<br>", tag_urls))
    }
}

pub fn blogpost(post: &Post, db: &PgConnection) -> AResult<String> {
    let mut html = render_markdown(&post.content);
    html += &tag_list(&post.url, db)?;
    Ok(format!(
        include_str!("skeleton.html"),
        body = html,
        title = post.title,
        copyright = copyright_years(&post.created, &post.updated),
        bottom_navigation = bottom_navigation(post, db)?,
    ))
}

fn tag_overview(db: &PgConnection) -> AResult<String> {
    use crate::schema::tags::dsl::*;

    let t = tags
        .distinct_on(tag)
        .order_by(tag)
        .load::<models::Tag>(db)?;

    let mut buf = String::from("<h1>Posts sorted by tags</h1>");

    buf += "<ul>";
    for t in t.into_iter() {
        buf += &format!("<li>{}</li>", t.link());
    }
    buf += "</ul>";

    Ok(buf)
}

pub fn overview(db: &PgConnection) -> AResult<String> {
    let mut body = String::from("<h1>Blog Posts</h1>");

    use crate::schema::posts::dsl::*;
    let sites = posts
        .filter(published)
        .order_by(created.desc())
        .select((title, url, created))
        .load(db)?;

    body += &create_table(&sites);
    //body += &tag_overview(db)?;

    body += &crate::polyring::BANNER;

    let page = format!(
        include_str!("skeleton.html"),
        title = "Overview",
        body = body,
        bottom_navigation = "",
        copyright = copyright_years(&sites.last().unwrap().2, &sites.first().unwrap().2)
    );

    Ok(page)
}

fn create_table(posts: &[(String, String, NaiveDateTime)]) -> String {
    let mut body = String::from("<hr>");
    body += r#"<table class="post-list">"#;
    body += "<th>Post</th><th>Date</th>";
    for (title, url, created) in posts.iter() {
        body += &format!(
            r#"<tr><td><a href="{}">{}</a></td><td>{}</td></tr>"#,
            PageKind::Post.url_of(&url),
            &title,
            created.date().format("%d-%m-%Y")
        );
    }
    body += "</table><hr>";
    body
}

pub fn tag(name: &str, db: &PgConnection) -> AResult<String> {
    let title = format!("Posts with tag {}", name.to_uppercase());
    let mut body = format!("<h1>{}</h1>", title);

    use crate::schema::posts::dsl as p;
    use crate::schema::tags::dsl as t;
    use crate::schema::tags_meta::dsl as m;
    let sites = t::tags
        .inner_join(p::posts.on(p::url.eq(t::url)))
        .filter(p::published)
        .filter(t::tag.eq(name))
        .order_by(p::created.desc())
        .select((p::title, p::url, p::created))
        .load(db)?;

    let description = m::tags_meta
        .filter(m::tag.eq(name))
        .select(m::description)
        .load::<String>(db)?;
    let description = description
        .first()
        .ok_or("not in meta table, something is wrong with db...")?;

    body += &render_markdown(&description);
    body += &create_table(&sites);

    let page = format!(
        include_str!("skeleton.html"),
        title = title,
        body = body,
        bottom_navigation = "",
        copyright = ""
    );
    Ok(page)
}
