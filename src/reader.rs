#[derive(Clone, Debug)]
pub enum Block {
    Heading(u8, String),
    Paragraph(String),
    Code(String),
    Quote(String),
    ListItem(String),
}

fn strip_inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
        .replace("&rsquo;", "\u{2019}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&ldquo;", "\u{201C}")
}

// Returns (blocks, section_line_offsets) where offsets are approximate line
// positions of each heading block (estimated at ~100 char width).
pub fn parse_html(html: &str) -> (Vec<Block>, Vec<usize>) {
    let mut blocks = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        let slice = html[pos..].trim_start();
        if slice.is_empty() {
            break;
        }
        pos = html.len() - slice.len();

        if !html[pos..].starts_with('<') {
            pos += html[pos..].find('<').unwrap_or(html.len() - pos);
            continue;
        }

        let tag_end = match html[pos..].find('>') {
            Some(i) => pos + i,
            None => break,
        };

        let tag_inner = &html[pos + 1..tag_end];
        let tag_name = tag_inner
            .split(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r')
            .next()
            .unwrap_or("")
            .to_lowercase();

        // Skip closing tags and comments/doctype
        if tag_name.starts_with('/') || tag_name.starts_with('!') {
            pos = tag_end + 1;
            continue;
        }

        // Self-closing: skip
        if tag_inner.ends_with('/') {
            pos = tag_end + 1;
            continue;
        }

        let content_start = tag_end + 1;

        match tag_name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag_name
                    .chars()
                    .nth(1)
                    .and_then(|c| c.to_digit(10))
                    .unwrap_or(1) as u8;
                let close = format!("</h{level}>");
                if let Some(rel) = html[content_start..].find(&close) {
                    let text = strip_inline(&html[content_start..content_start + rel]);
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block::Heading(level, text));
                    }
                    pos = content_start + rel + close.len();
                } else {
                    pos = content_start;
                }
            }
            "p" => {
                if let Some(rel) = html[content_start..].find("</p>") {
                    let text = strip_inline(&html[content_start..content_start + rel]);
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block::Paragraph(text));
                    }
                    pos = content_start + rel + 4;
                } else {
                    pos = content_start;
                }
            }
            "pre" => {
                if let Some(rel) = html[content_start..].find("</pre>") {
                    let text = strip_inline(&html[content_start..content_start + rel]);
                    if !text.trim().is_empty() {
                        blocks.push(Block::Code(text));
                    }
                    pos = content_start + rel + 6;
                } else {
                    pos = content_start;
                }
            }
            "blockquote" => {
                if let Some(rel) = html[content_start..].find("</blockquote>") {
                    let text = strip_inline(&html[content_start..content_start + rel]);
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block::Quote(text));
                    }
                    pos = content_start + rel + 13;
                } else {
                    pos = content_start;
                }
            }
            "li" => {
                if let Some(rel) = html[content_start..].find("</li>") {
                    let text = strip_inline(&html[content_start..content_start + rel]);
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        blocks.push(Block::ListItem(text));
                    }
                    pos = content_start + rel + 5;
                } else {
                    pos = content_start;
                }
            }
            _ => {
                // Container/inline tags: skip opening tag, parse inner content
                pos = content_start;
            }
        }
    }

    // Approximate line offsets for section navigation (assumes ~100 char width)
    let mut section_offsets = Vec::new();
    let mut line = 0usize;
    for block in &blocks {
        match block {
            Block::Heading(_, _) => {
                section_offsets.push(line);
                line += 2;
            }
            Block::Paragraph(text) => {
                line += (text.len() / 100).max(1) + 1;
            }
            Block::Code(text) => {
                line += text.lines().count() + 1;
            }
            Block::Quote(text) => {
                line += text.lines().count() + 1;
            }
            Block::ListItem(_) => {
                line += 1;
            }
        }
    }

    (blocks, section_offsets)
}
