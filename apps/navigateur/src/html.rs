//! Petit parseur HTML tolérant : produit une liste plate d'éléments (mots, sauts, images) prête à être mise en page.

#[derive(Clone, Debug)]
pub enum Item {
    Word { text: String, bold: bool, mono: bool, head: u8, link: Option<usize> },
    Br,
    Para,
    Rule,
    Image { src: String, alt: String, link: Option<usize> },
}

#[derive(Default)]
pub struct Doc {
    pub items: Vec<Item>,
    pub links: Vec<String>,
    pub title: String,
}

const SKIP: &[&str] = &[
    "script", "style", "noscript", "svg", "template", "iframe", "select", "textarea", "object", "canvas", "video", "audio", "map",
];
const BLOCK: &[&str] = &[
    "p", "div", "section", "article", "header", "footer", "nav", "main", "aside", "table", "blockquote", "form", "fieldset", "figure",
    "figcaption", "tr", "dl", "dt", "dd", "address", "details", "summary", "center",
];

fn starts(c: &[char], i: usize, pat: &str) -> bool {
    let mut k = i;
    for p in pat.chars() {
        if c.get(k).copied() != Some(p) {
            return false;
        }
        k += 1;
    }
    true
}

fn find(c: &[char], from: usize, pat: &str) -> Option<usize> {
    let pl = pat.chars().count();
    if c.len() < pl {
        return None;
    }
    (from..=c.len() - pl).find(|&i| starts(c, i, pat))
}

fn find_ci(c: &[char], from: usize, pat: &str) -> Option<usize> {
    let p: Vec<char> = pat.chars().collect();
    if c.len() < p.len() {
        return None;
    }
    (from..=c.len() - p.len()).find(|&i| p.iter().enumerate().all(|(k, &pc)| c[i + k].to_ascii_lowercase() == pc))
}

fn para(items: &mut Vec<Item>) {
    if let Some(last) = items.last() {
        if !matches!(last, Item::Para) {
            items.push(Item::Para);
        }
    }
}

fn br(items: &mut Vec<Item>) {
    if let Some(last) = items.last() {
        if !matches!(last, Item::Para | Item::Br) {
            items.push(Item::Br);
        }
    }
}

/// Valeur d'un attribut dans la chaîne d'attributs d'une balise.
fn attr(attrs: &str, key: &str) -> Option<String> {
    let c: Vec<char> = attrs.chars().collect();
    let n = c.len();
    let mut i = 0;
    while i < n {
        while i < n && (c[i].is_whitespace() || c[i] == '/') {
            i += 1;
        }
        let s = i;
        while i < n && !c[i].is_whitespace() && c[i] != '=' {
            i += 1;
        }
        let name: String = c[s..i].iter().collect::<String>().to_ascii_lowercase();
        while i < n && c[i].is_whitespace() {
            i += 1;
        }
        let mut val = String::new();
        if i < n && c[i] == '=' {
            i += 1;
            while i < n && c[i].is_whitespace() {
                i += 1;
            }
            if i < n && (c[i] == '"' || c[i] == '\'') {
                let q = c[i];
                i += 1;
                let vs = i;
                while i < n && c[i] != q {
                    i += 1;
                }
                val = c[vs..i].iter().collect();
                i += 1;
            } else {
                let vs = i;
                while i < n && !c[i].is_whitespace() {
                    i += 1;
                }
                val = c[vs..i].iter().collect();
            }
        }
        if name == key {
            return Some(decode(&val));
        }
    }
    None
}

pub fn decode(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        rest = &rest[p..];
        let semi = rest.char_indices().take(12).find(|(_, ch)| *ch == ';').map(|(i, _)| i);
        let mut done = false;
        if let Some(e) = semi {
            let ent = &rest[1..e];
            let ch = match ent {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                "ndash" => Some('–'),
                "mdash" => Some('—'),
                "hellip" => Some('…'),
                "laquo" => Some('«'),
                "raquo" => Some('»'),
                "copy" => Some('©'),
                "reg" => Some('®'),
                "rsquo" | "lsquo" => Some('\''),
                "rdquo" | "ldquo" => Some('"'),
                "bull" | "middot" => Some('·'),
                "euro" => Some('€'),
                "times" => Some('×'),
                "eacute" => Some('é'),
                "egrave" => Some('è'),
                "ecirc" => Some('ê'),
                "agrave" => Some('à'),
                "acirc" => Some('â'),
                "ccedil" => Some('ç'),
                "ocirc" => Some('ô'),
                "icirc" => Some('î'),
                "ucirc" => Some('û'),
                _ if ent.starts_with('#') => {
                    let num = &ent[1..];
                    let v = if let Some(h) = num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
                        u32::from_str_radix(h, 16).ok()
                    } else {
                        num.parse::<u32>().ok()
                    };
                    v.and_then(char::from_u32)
                }
                _ => None,
            };
            if let Some(ch) = ch {
                out.push(ch);
                rest = &rest[e + 1..];
                done = true;
            }
        }
        if !done {
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

pub fn parse(src: &str) -> Doc {
    let c: Vec<char> = src.chars().collect();
    let n = c.len();
    let mut d = Doc::default();
    let (mut bold, mut mono, mut pre) = (0i32, 0i32, 0i32);
    let mut head: u8 = 0;
    let mut link: Option<usize> = None;
    let (mut in_title, mut in_head) = (false, false);
    let mut lists: Vec<Option<usize>> = Vec::new();
    let mut i = 0;
    while i < n && d.items.len() < 60000 {
        if c[i] == '<' {
            if starts(&c, i, "<!--") {
                i = find(&c, i + 4, "-->").map(|p| p + 3).unwrap_or(n);
                continue;
            }
            let next = c.get(i + 1).copied().unwrap_or(' ');
            if next == '!' || next == '?' {
                i = c[i..].iter().position(|&x| x == '>').map(|p| i + p + 1).unwrap_or(n);
                continue;
            }
            if next == '/' || next.is_ascii_alphabetic() {
                // fin de balise, en respectant les guillemets (avec repli sur le premier '>')
                let limit = (i + 4000).min(n);
                let mut j = i + 1;
                let mut q: Option<char> = None;
                while j < limit {
                    let ch = c[j];
                    match q {
                        Some(qc) => {
                            if ch == qc {
                                q = None;
                            }
                        }
                        None => {
                            if ch == '"' || ch == '\'' {
                                q = Some(ch);
                            } else if ch == '>' {
                                break;
                            }
                        }
                    }
                    j += 1;
                }
                if j >= limit || c[j] != '>' {
                    j = c[i..].iter().position(|&x| x == '>').map(|p| i + p).unwrap_or(n.saturating_sub(1));
                }
                let tag: String = c[i + 1..j.min(n)].iter().collect();
                i = (j + 1).min(n);
                let closing = tag.starts_with('/');
                let selfclose = tag.trim_end().ends_with('/');
                let body = tag.trim_start_matches('/');
                let name_end = body.find(|ch: char| !ch.is_ascii_alphanumeric()).unwrap_or(body.len());
                let name = body[..name_end].to_ascii_lowercase();
                let attrs = &body[name_end..];

                if !closing && !selfclose && SKIP.contains(&name.as_str()) {
                    i = find_ci(&c, i, &format!("</{name}")).unwrap_or(n);
                    continue;
                }
                match name.as_str() {
                    "title" => in_title = !closing,
                    "head" => in_head = !closing,
                    "body" => in_head = false,
                    "br" => {
                        if !matches!(d.items.last(), Some(Item::Para) | None) {
                            d.items.push(Item::Br);
                        }
                    }
                    "hr" => {
                        para(&mut d.items);
                        d.items.push(Item::Rule);
                        d.items.push(Item::Para);
                    }
                    "a" => {
                        if closing {
                            link = None;
                        } else if let Some(h) = attr(attrs, "href") {
                            let h = h.trim().to_string();
                            let l = h.to_ascii_lowercase();
                            if h.is_empty() || h.starts_with('#') || l.starts_with("javascript:") || l.starts_with("mailto:") || l.starts_with("tel:") {
                                link = None;
                            } else {
                                d.links.push(h);
                                link = Some(d.links.len() - 1);
                            }
                        }
                    }
                    "img" if !closing => {
                        let src = attr(attrs, "src").filter(|s| !s.trim().is_empty()).or_else(|| attr(attrs, "data-src"));
                        let tiny = attr(attrs, "width").and_then(|w| w.trim_end_matches("px").parse::<f32>().ok()).map_or(false, |w| w < 20.0)
                            || attr(attrs, "height").and_then(|w| w.trim_end_matches("px").parse::<f32>().ok()).map_or(false, |h| h < 20.0);
                        if let Some(src) = src {
                            if !src.starts_with("data:") && !tiny {
                                let alt = attr(attrs, "alt").unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" ");
                                d.items.push(Item::Image { src, alt, link });
                            }
                        }
                    }
                    "b" | "strong" | "th" => bold = (bold + if closing { -1 } else { 1 }).max(0),
                    "code" | "tt" | "kbd" | "samp" => mono = (mono + if closing { -1 } else { 1 }).max(0),
                    "pre" => {
                        para(&mut d.items);
                        if closing {
                            pre = (pre - 1).max(0);
                            mono = (mono - 1).max(0);
                        } else {
                            pre += 1;
                            mono += 1;
                        }
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        para(&mut d.items);
                        if closing {
                            head = 0;
                            bold = (bold - 1).max(0);
                        } else {
                            head = name.as_bytes()[1] - b'0';
                            bold += 1;
                        }
                    }
                    "ul" | "ol" => {
                        para(&mut d.items);
                        if closing {
                            lists.pop();
                        } else {
                            lists.push(if name == "ol" { Some(0) } else { None });
                        }
                    }
                    "li" if !closing => {
                        br(&mut d.items);
                        let marker = match lists.last_mut() {
                            Some(Some(cnt)) => {
                                *cnt += 1;
                                format!("{cnt}.")
                            }
                            _ => "•".to_string(),
                        };
                        d.items.push(Item::Word { text: marker, bold: false, mono: false, head: 0, link: None });
                    }
                    n if BLOCK.contains(&n) => para(&mut d.items),
                    _ => {}
                }
                continue;
            }
        }
        // texte (le premier caractère est consommé d'office : un '<' isolé est du texte)
        let start = i;
        i += 1;
        while i < n && c[i] != '<' {
            i += 1;
        }
        let raw: String = c[start..i].iter().collect();
        let t = decode(&raw);
        if in_title {
            d.title.push(' ');
            d.title.push_str(&t);
        } else if !in_head {
            if pre > 0 {
                for (k, line) in t.split('\n').enumerate() {
                    if k > 0 {
                        d.items.push(Item::Br);
                    }
                    for w in line.split_whitespace() {
                        d.items.push(Item::Word { text: w.to_string(), bold: bold > 0, mono: true, head, link });
                    }
                }
            } else {
                for w in t.split_whitespace() {
                    d.items.push(Item::Word { text: w.to_string(), bold: bold > 0, mono: mono > 0, head, link });
                }
            }
        }
    }
    d.title = d.title.split_whitespace().collect::<Vec<_>>().join(" ");
    d
}