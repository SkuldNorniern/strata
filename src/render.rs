use std::path::Path;
use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use crate::fs_utils::escape_attr;
use crate::types::WikiError;

/// Render Markdown content with TOC generation and heading anchors
pub fn render_markdown_with_toc(path: &Path) -> Result<(String, String, Option<String>), WikiError> {
    let raw = std::fs::read_to_string(path)?;
    let (front_title, content) = split_front_matter(&raw);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    // First pass: collect headings
    let mut headings: Vec<(u32, String, String)> = Vec::new(); // (level, id, text)
    let mut in_heading: Option<u32> = None;
    let mut buf = String::new();
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for ev in Parser::new_ext(&content, options) {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = Some(heading_level_to_u32(level));
                buf.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(lvl) = in_heading.take() {
                    let mut id = slugify(&buf);
                    if id.is_empty() { id = format!("h{}", lvl); }
                    let count = id_counts.entry(id.clone()).or_insert(0);
                    if *count > 0 { id = format!("{}-{}", id, *count); }
                    *count += 1;
                    headings.push((lvl, id, buf.clone()));
                }
                buf.clear();
            }
            Event::Text(t) | Event::Code(t) => {
                if in_heading.is_some() { buf.push_str(&t); }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_heading.is_some() { buf.push(' '); }
            }
            _ => {}
        }
    }

    // Second pass: inject ids and heading anchors
    let mut out = String::new();
    let mut idx = 0usize;
    let mut closing_stack: Vec<u32> = Vec::new();
    let mut id_stack: Vec<String> = Vec::new();
    for ev in Parser::new_ext(&content, options) {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = heading_level_to_u32(level);
                let id = headings.get(idx).map(|(_, id, _)| id.as_str()).unwrap_or("");
                out.push_str(&format!("<h{} id=\"{}\">", lvl, escape_attr(id)));
                closing_stack.push(lvl);
                id_stack.push(id.to_string());
                idx += 1;
            }
            Event::End(TagEnd::Heading(_)) => {
                let id_for_anchor = id_stack.pop().unwrap_or_default();
                out.push_str(&format!("<a class=\"hlink\" href=\"#{}\" aria-label=\"Link to this section\">#</a>", escape_attr(&id_for_anchor)));
                if let Some(lvl) = closing_stack.pop() {
                    out.push_str(&format!("</h{}>", lvl));
                } else {
                    out.push_str("</h1>");
                }
            }
            _ => html::push_html(&mut out, std::iter::once(ev)),
        }
    }

    let toc = build_toc_html(&headings);
    let page_title = front_title.or_else(|| first_heading_text(&headings));
    Ok((out, toc, page_title))
}

/// Build HTML for the Table of Contents
fn build_toc_html(headings: &[(u32, String, String)]) -> String {
    if headings.is_empty() { return String::new(); }
    let mut html = String::new();
    html.push_str("<nav class=\"toc\"><div class=\"toc-title\">Contents</div>");
    let mut current = 0u32;
    for (level, id, title) in headings {
        if *level > 6 || *level < 1 { continue; }
        while current < *level { html.push_str("<ul>"); current += 1; }
        while current > *level { html.push_str("</ul>"); current -= 1; }
        html.push_str(&format!("<li><a href=\"#{}\">{}</a></li>", escape_attr(id), title));
    }
    while current > 0 { html.push_str("</ul>"); current -= 1; }
    html.push_str("</nav>");
    html
}

/// Convert heading level to u32
fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Create URL-friendly slug from text
fn slugify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_dash = false;
    for ch in text.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_dash = false;
        } else if c.is_ascii_whitespace() || c == '-' || c == '_' {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        }
    }
    if out.ends_with('-') { out.pop(); }
    out
}

/// Extract front matter from Markdown content
fn split_front_matter(raw: &str) -> (Option<String>, &str) {
    let bytes = raw.as_bytes();
    if raw.starts_with("---\n") {
        // find closing ---\n
        let mut idx = 4; // after first line
        let mut title: Option<String> = None;
        while idx < bytes.len() {
            // find line end
            let start = idx;
            while idx < bytes.len() && bytes[idx] != b'\n' { idx += 1; }
            let line = &raw[start..idx];
            if line.trim() == "---" {
                let body = &raw[idx+1..];
                return (title, body);
            }
            if let Some((k, v)) = line.split_once(':') {
                if k.trim().eq_ignore_ascii_case("title") {
                    let mut val = v.trim();
                    if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                        val = &val[1..val.len()-1];
                    }
                    if !val.is_empty() { title = Some(val.to_string()); }
                }
            }
            idx += 1; // skip newline
        }
    }
    (None, raw)
}

/// Get the first H1 heading text
fn first_heading_text(headings: &[(u32, String, String)]) -> Option<String> {
    for (lvl, _id, text) in headings {
        if *lvl == 1 && !text.trim().is_empty() {
            return Some(text.clone());
        }
    }
    None
}


