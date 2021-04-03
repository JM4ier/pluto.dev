use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, html::*, parsing::SyntaxSet,
    util::LinesWithEndings,
};

use std::error::Error;

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
