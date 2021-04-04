use lazy_static::lazy_static;
use std::error::Error;
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, html::*, parsing::SyntaxSet,
    util::LinesWithEndings,
};

lazy_static! {
    static ref SYNTAX: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME: ThemeSet = ThemeSet::load_defaults();
}

pub fn highlight(code: &str, lang: &str) -> Result<String, Box<dyn Error>> {
    let syntax = SYNTAX
        .find_syntax_by_name(lang)
        .ok_or(format!("syntax \"{}\" not known", lang))?;

    let mut h = HighlightLines::new(syntax, &THEME.themes["base16-ocean.dark"]);

    let mut buf = String::new();
    for line in LinesWithEndings::from(code) {
        let ranges = h.highlight(line, &SYNTAX);
        buf += &styled_line_to_highlighted_html(&ranges, IncludeBackground::No);
    }

    Ok(buf)
}
