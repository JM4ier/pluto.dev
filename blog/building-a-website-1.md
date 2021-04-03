---
title: Building a website from scratch (Part 1/?)
published: true
version: 1
---

# Building a website from Scratch
This serves as my first real post on this site, and fittingly, it is about the site itself.

---

I don't remember my exact motivation behind it, but I decided to start building a website from scratch.
As all my recent projects have been written in [Rust](https://www.rust-lang.org/), I decided to use the language for this project as well.
Also I'm fairly lazy, so I'm not writing the content in pure html, but instead use a markdown parser that produces that for me.

The initial setup looked something like this:
```
use pulldown_cmark::*;

mod code;

fn main() {
   let markdown_input = include_str!("content.md");
   let parser = Parser::new(markdown_input);

   let mut html_out = String::new();
   html::push_html(&mut html_out, parser);

   println!(
      include_str!("skeleton.html"),
      body = html_out,
      title = "Website"
   );
}
```

This just converts the markdown into pure, black and white html, using the wonderful [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) markdown parser, and puts it into a "skeleton" html, which contains just the bare bones (html, head, body tags), then prints it.

Then I started putting together a basic dark theme, as I don't really like pure white backgrounds.
This is when I ran into a different issue: with the current setup, I can't have syntax highlighting in my code, unless I include it as an image or run some javascript in the browser.
Since I don't like either, I decided to generate html for it as well.

Luckily the Rusts crate registry, [crates.io](https://crates.io), has a lot of awesome libraries, like [syntect](https://github.com/trishume/syntect), which is an easy to use library to apply syntax highlighting to code.

After playing around with it a bit, I found a nice way to adjust the markdown parser to output highlighted code:

```rs
// in the main function
use Event::*;

let mut code_lang = None;
let parser = parser.flat_map(|event| {
   match &event {

      // finds start of code blocks and begins div
      Start(Tag::CodeBlock(kind)) => {
         if let CodeBlockKind::Fenced(kind) = kind {
            code_lang = Some(kind.clone().into_string());
         }
         vec![Html(r#"<div class="code">"#.into()), event]
      }

      // colors text if currently in code block
      Text(code) => code_lang
         .as_ref()
         .map(|lang| highlight(&code, &lang).ok())
         .flatten()
         .map_or(vec![event], |html| vec![Html(html.into())]),

      // finds end of code blocks and ends div
      End(Tag::CodeBlock(_)) => {
         code_lang = None;
         vec![event, Html("</div>".into())]
      }

      // keep other markdown elements
      _ => {
         vec![event]
      }
   }
});
```
```rs
// example code almost copied one to one from syntect docs
pub fn highlight(code: &str, lang: &str) -> Result<String, Box<dyn Error>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps
        .find_syntax_by_extension(lang)
        .ok_or(format!("syntax \"{}\" not known", lang))?;

    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut buf = String::new();
    for line in LinesWithEndings::from(code) {
        let ranges = h.highlight(line, &ps);
        buf += &styled_line_to_highlighted_html(&ranges, IncludeBackground::No);
    }

    Ok(buf)
}
```

This injects the needed html tags for the syntax highlighting by adding raw html tags before respectively after the code blocks start and end. All the text between code block start & end tags is put through the syntax `highlight` function.

I am rather happy with this result, as such little code, markdown, and css produces a decent looking site. 

---

This pretty much concludes my first work on the website rendered/builder.

Since then, I added a database to link pages easily, but that is topic for a different post.


