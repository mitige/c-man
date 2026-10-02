//! Mini-parseur markdown (sous-ensemble) partagé par le rendu TUI et le rendu ANSI.
//!
//! Supporte : `#`/`##`/`###` titres, `-` listes, `>` citations, `**gras**`,
//! `` `code` `` inline, blocs fenced ``` (rendus comme code brut).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seg {
    Normal,
    Bold,
    Code,
    Dim,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdLine {
    /// Ligne de texte avec segments stylés (à wrapper).
    Text(Vec<(Seg, String)>),
    /// Titre de niveau 1..=3.
    Heading(u8, String),
    /// Puce de liste.
    Bullet(Vec<(Seg, String)>),
    /// Élément de liste numérotée (numéro conservé).
    BulletNum(String, Vec<(Seg, String)>),
    /// Citation / note.
    Quote(Vec<(Seg, String)>),
    /// Ligne de code brute (coloration syntaxique à part).
    Code(String),
    Blank,
}

/// Découpe une chaîne en segments **gras** / `code` / normal.
pub fn parse_inline(text: &str) -> Vec<(Seg, String)> {
    let mut segs: Vec<(Seg, String)> = Vec::new();
    let mut rest = text;
    let mut push = |seg: Seg, s: &str| {
        if s.is_empty() {
            return;
        }
        if let Some(last) = segs.last_mut() {
            if last.0 == seg {
                last.1.push_str(s);
                return;
            }
        }
        segs.push((seg, s.to_string()));
    };
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix("**") {
            match stripped.find("**") {
                Some(end) if end > 0 => {
                    push(Seg::Bold, &stripped[..end]);
                    rest = &stripped[end + 2..];
                }
                _ => {
                    push(Seg::Normal, "**");
                    rest = stripped;
                }
            }
        } else if let Some(stripped) = rest.strip_prefix('`') {
            match stripped.find('`') {
                Some(end) if end > 0 => {
                    push(Seg::Code, &stripped[..end]);
                    rest = &stripped[end + 1..];
                }
                _ => {
                    push(Seg::Normal, "`");
                    rest = stripped;
                }
            }
        } else {
            let next = rest.find(['*', '`']).unwrap_or(rest.len());
            if next == 0 {
                // '*' isolé (pas un marqueur **) : caractère littéral
                push(Seg::Normal, "*");
                rest = &rest[1..];
            } else {
                push(Seg::Normal, &rest[..next]);
                rest = &rest[next..];
            }
        }
    }
    segs
}

/// Parse un document markdown en lignes typées.
pub fn parse(md: &str) -> Vec<MdLine> {
    let mut out = Vec::new();
    let mut in_code = false;
    let mut para: Vec<String> = Vec::new();

    let flush_para = |out: &mut Vec<MdLine>, para: &mut Vec<String>| {
        if !para.is_empty() {
            out.push(MdLine::Text(parse_inline(&para.join(" "))));
            para.clear();
        }
    };

    for raw in md.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            flush_para(&mut out, &mut para);
            in_code = !in_code;
            continue;
        }
        if in_code {
            out.push(MdLine::Code(raw.trim_end().to_string()));
            continue;
        }
        let t = line.trim();
        if t.is_empty() {
            flush_para(&mut out, &mut para);
            out.push(MdLine::Blank);
        } else if let Some(h) = t.strip_prefix("### ") {
            flush_para(&mut out, &mut para);
            out.push(MdLine::Heading(3, h.to_string()));
        } else if let Some(h) = t.strip_prefix("## ") {
            flush_para(&mut out, &mut para);
            out.push(MdLine::Heading(2, h.to_string()));
        } else if let Some(h) = t.strip_prefix("# ") {
            flush_para(&mut out, &mut para);
            out.push(MdLine::Heading(1, h.to_string()));
        } else if let Some(b) = t.strip_prefix("- ") {
            flush_para(&mut out, &mut para);
            out.push(MdLine::Bullet(parse_inline(b)));
        } else if t
            .find(". ")
            .is_some_and(|pos| pos <= 2 && t[..pos].chars().all(|c| c.is_ascii_digit()) && !t[..pos].is_empty())
        {
            // liste numérotée : "1. texte" — conserve le numéro comme puce
            flush_para(&mut out, &mut para);
            let pos = t.find(". ").unwrap();
            out.push(MdLine::BulletNum(
                format!("{}.", &t[..pos]),
                parse_inline(&t[pos + 2..]),
            ));
        } else if let Some(q) = t.strip_prefix("> ") {
            flush_para(&mut out, &mut para);
            out.push(MdLine::Quote(parse_inline(q)));
        } else {
            para.push(t.to_string());
        }
    }
    flush_para(&mut out, &mut para);
    // retire les blancs en fin
    while matches!(out.last(), Some(MdLine::Blank)) {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_bold_and_code() {
        let segs = parse_inline("du **gras** et du `code` ici");
        assert!(segs.contains(&(Seg::Bold, "gras".to_string())));
        assert!(segs.contains(&(Seg::Code, "code".to_string())));
    }

    #[test]
    fn inline_lone_star_terminates() {
        // régression : un '*' isolé (déréférencement C) bouclait à l'infini
        let segs = parse_inline("un *p et *x isolés");
        let joined: String = segs.iter().map(|(_, s)| s.as_str()).collect();
        assert_eq!(joined, "un *p et *x isolés");
    }

    #[test]
    fn inline_unmatched_marker_is_literal() {
        let segs = parse_inline("un ** non fermé");
        let joined: String = segs.iter().map(|(_, s)| s.as_str()).collect();
        assert_eq!(joined, "un ** non fermé");
    }

    #[test]
    fn parse_blocks() {
        let md = "## Titre\n\nUn paragraphe\nsur deux lignes.\n\n- puce\n- puce2\n\n```c\nint x = 1;\n```\n\n> note";
        let lines = parse(md);
        assert!(lines.contains(&MdLine::Heading(2, "Titre".to_string())));
        assert!(lines.contains(&MdLine::Code("int x = 1;".to_string())));
        assert!(matches!(
            lines.iter().find(|l| matches!(l, MdLine::Quote(_))),
            Some(_)
        ));
        let bullets = lines
            .iter()
            .filter(|l| matches!(l, MdLine::Bullet(_)))
            .count();
        assert_eq!(bullets, 2);
    }

    #[test]
    fn code_fence_preserves_stars() {
        let md = "```c\nchar **argv;\n```";
        let lines = parse(md);
        assert_eq!(lines, vec![MdLine::Code("char **argv;".to_string())]);
    }
}
