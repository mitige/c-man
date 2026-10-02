//! Rendu d'une fiche directement sur stdout (mode `c-man <sujet>`).

use crate::content::{category_label, Entry};
use crate::highlight;
use crate::markdown::{parse, MdLine, Seg};
use unicode_width::UnicodeWidthStr;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[38;5;80m";
const GOLD: &str = "\x1b[38;5;220m";
const GREEN: &str = "\x1b[38;5;78m";
const CODE_FG: &str = "\x1b[38;5;222m";
const PURPLE: &str = "\x1b[38;5;141m";

pub struct Renderer {
    pub color: bool,
    pub width: usize,
}

fn style_on(color: bool, code: &str) -> &str {
    if color {
        code
    } else {
        ""
    }
}

fn reset(color: bool) -> &'static str {
    if color {
        RESET
    } else {
        ""
    }
}

/// Largeur visible d'une chaîne.
fn w(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Découpe des segments stylés en lignes wrappées à `width` colonnes.
pub fn wrap_segments(segs: &[(Seg, String)], width: usize) -> Vec<Vec<(Seg, String)>> {
    let mut lines: Vec<Vec<(Seg, String)>> = vec![Vec::new()];
    let mut col = 0usize;
    for (seg, text) in segs {
        for word in text.split_inclusive(char::is_whitespace) {
            let trimmed = word.trim_end();
            let spaces = word.len() - trimmed.len();
            let ww = w(trimmed);
            if !trimmed.is_empty() {
                if col > 0 && col + ww > width {
                    lines.push(Vec::new());
                    col = 0;
                }
                lines.last_mut().unwrap().push((*seg, trimmed.to_string()));
                col += ww;
            }
            if spaces > 0 && col > 0 {
                if col + 1 > width {
                    lines.push(Vec::new());
                    col = 0;
                } else {
                    if let Some(last) = lines.last_mut().unwrap().last_mut() {
                        last.1.push(' ');
                    }
                    col += 1;
                }
            }
        }
    }
    lines
}

impl Renderer {
    fn seg_to_ansi(&self, seg: Seg, text: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        match seg {
            Seg::Normal => text.to_string(),
            Seg::Bold => format!("{BOLD}{text}{RESET}"),
            Seg::Code => format!("{CODE_FG}{text}{RESET}"),
            Seg::Dim => format!("{DIM}{text}{RESET}"),
        }
    }

    fn emit_wrapped(&self, out: &mut String, segs: &[(Seg, String)], indent: &str, width: usize) {
        for line in wrap_segments(segs, width) {
            out.push_str(indent);
            for (seg, text) in &line {
                out.push_str(&self.seg_to_ansi(*seg, text));
            }
            out.push('\n');
        }
    }

    fn rule(&self, out: &mut String, title: &str, color_code: &str) {
        let inner = self.width.saturating_sub(2);
        let label = format!("─ {title} ");
        let fill = inner.saturating_sub(w(&label));
        out.push_str(&format!(
            "{c}┌{label}{fill}┐{r}\n",
            c = style_on(self.color, color_code),
            fill = "─".repeat(fill),
            r = reset(self.color),
        ));
    }

    fn box_bottom(&self, out: &mut String, color_code: &str) {
        let inner = self.width.saturating_sub(2);
        out.push_str(&format!(
            "{c}└{fill}┘{r}\n",
            c = style_on(self.color, color_code),
            fill = "─".repeat(inner),
            r = reset(self.color),
        ));
    }

    fn box_line(&self, out: &mut String, content: &str, visible_w: usize, color_code: &str) {
        let inner = self.width.saturating_sub(4);
        let pad = inner.saturating_sub(visible_w);
        out.push_str(&format!(
            "{c}│{r} {content}{pad} {c}│{r}\n",
            c = style_on(self.color, color_code),
            r = reset(self.color),
            pad = " ".repeat(pad),
        ));
    }

    /// Rend la fiche complète.
    pub fn render_entry(&self, entry: &Entry) -> String {
        let mut out = String::new();
        let width = self.width.clamp(48, 160);

        // En-tête
        out.push('\n');
        out.push_str(&format!(
            "{b}{c}  {title}{r}\n",
            b = style_on(self.color, BOLD),
            c = style_on(self.color, CYAN),
            title = entry.title,
            r = reset(self.color),
        ));
        let mut meta = format!("  {}", category_label(&entry.category));
        if let Some(day) = &entry.piscine_day {
            meta.push_str(&format!(" · piscine {day}"));
        }
        if !entry.tags.is_empty() {
            meta.push_str(&format!(" · #{}", entry.tags.join(" #")));
        }
        out.push_str(&format!(
            "{d}{meta}{r}\n\n",
            d = style_on(self.color, DIM),
            r = reset(self.color),
        ));

        // Synopsis
        self.rule(&mut out, "SYNOPSIS", CYAN);
        let inner = width.saturating_sub(5);
        for line in entry.synopsis.lines() {
            let truncated: String = if w(line) > inner {
                let mut s: String = line.chars().take(inner.saturating_sub(1)).collect();
                s.push('…');
                s
            } else {
                line.to_string()
            };
            let vw = w(&truncated);
            let content = if self.color {
                format!("{BOLD}{truncated}{RESET}")
            } else {
                truncated
            };
            self.box_line(&mut out, &content, vw, CYAN);
        }
        self.box_bottom(&mut out, CYAN);
        out.push('\n');

        // Description
        let md = parse(&entry.description);
        self.render_md(&mut out, &md, width);
        out.push('\n');

        // Exemple
        self.rule(&mut out, "EXEMPLE", GREEN);
        let highlighted = highlight::highlight_c(&entry.example);
        let code_inner = width.saturating_sub(5);
        for line in &highlighted {
            let text: String = line.iter().map(|(_, t)| t.as_str()).collect();
            let vw = w(&text);
            if vw > code_inner {
                let mut s: String = text.chars().take(code_inner.saturating_sub(1)).collect();
                s.push('…');
                let svw = w(&s);
                self.box_line(&mut out, &s, svw, GREEN);
            } else if self.color {
                let ansi = highlight::line_to_ansi(line);
                self.box_line(&mut out, &ansi, vw, GREEN);
            } else {
                self.box_line(&mut out, &text, vw, GREEN);
            }
        }
        self.box_bottom(&mut out, GREEN);
        out.push('\n');

        // Pièges
        if !entry.gotchas.is_empty() {
            self.rule(&mut out, "PIÈGES", GOLD);
            for g in &entry.gotchas {
                let segs = vec![(Seg::Normal, g.clone())];
                let wrapped = wrap_segments(&segs, width.saturating_sub(8));
                for (i, line) in wrapped.iter().enumerate() {
                    let marker = if i == 0 { "⚠ " } else { "  " };
                    out.push_str(&format!(
                        "{c}{marker}{r}",
                        c = style_on(self.color, GOLD),
                        r = reset(self.color),
                    ));
                    for (seg, text) in line {
                        out.push_str(&self.seg_to_ansi(*seg, text));
                    }
                    out.push('\n');
                }
            }
            self.box_bottom(&mut out, GOLD);
            out.push('\n');
        }

        // Exercices
        if !entry.exercises.is_empty() {
            self.rule(&mut out, "EXERCICES", GREEN);
            for (i, e) in entry.exercises.iter().enumerate() {
                let segs = vec![(Seg::Normal, e.clone())];
                let wrapped = wrap_segments(&segs, width.saturating_sub(8));
                for (j, line) in wrapped.iter().enumerate() {
                    let marker = if j == 0 {
                        format!("{}. ", i + 1)
                    } else {
                        "   ".to_string()
                    };
                    out.push_str(&format!(
                        "{c}{marker}{r}",
                        c = style_on(self.color, GREEN),
                        r = reset(self.color),
                    ));
                    for (seg, text) in line {
                        out.push_str(&self.seg_to_ansi(*seg, text));
                    }
                    out.push('\n');
                }
            }
            self.box_bottom(&mut out, GREEN);
            out.push('\n');
        }

        // Voir aussi
        if !entry.related.is_empty() {
            out.push_str(&format!(
                "{p}→ Voir aussi :{r} ",
                p = style_on(self.color, PURPLE),
                r = reset(self.color),
            ));
            out.push_str(&entry.related.join(" · "));
            out.push('\n');
        }
        out.push('\n');
        out
    }

    fn render_md(&self, out: &mut String, lines: &[MdLine], width: usize) {
        let text_w = width.saturating_sub(4);
        for line in lines {
            match line {
                MdLine::Blank => out.push('\n'),
                MdLine::Heading(level, text) => {
                    let (pre, code) = match level {
                        1 => ("# ", PURPLE),
                        2 => ("## ", CYAN),
                        _ => ("### ", DIM),
                    };
                    out.push_str(&format!(
                        "{b}{c}{pre}{text}{r}\n",
                        b = style_on(self.color, BOLD),
                        c = style_on(self.color, code),
                        r = reset(self.color),
                    ));
                }
                MdLine::Text(segs) => self.emit_wrapped(out, segs, "", text_w),
                MdLine::Bullet(segs) => {
                    let wrapped = wrap_segments(segs, text_w.saturating_sub(2));
                    for (i, l) in wrapped.iter().enumerate() {
                        out.push_str(if i == 0 { "• " } else { "  " });
                        for (seg, text) in l {
                            out.push_str(&self.seg_to_ansi(*seg, text));
                        }
                        out.push('\n');
                    }
                }
                MdLine::BulletNum(num, segs) => {
                    let marker = format!("{num} ");
                    let indent = " ".repeat(marker.len());
                    let wrapped = wrap_segments(segs, text_w.saturating_sub(marker.len()));
                    for (i, l) in wrapped.iter().enumerate() {
                        let m = if i == 0 { marker.as_str() } else { indent.as_str() };
                        out.push_str(&format!(
                            "{c}{m}{r}",
                            c = style_on(self.color, CYAN),
                            r = reset(self.color),
                        ));
                        for (seg, text) in l {
                            out.push_str(&self.seg_to_ansi(*seg, text));
                        }
                        out.push('\n');
                    }
                }
                MdLine::Quote(segs) => {
                    let mut with_marker = vec![(Seg::Dim, "│ ".to_string())];
                    with_marker.extend(segs.iter().cloned());
                    self.emit_wrapped(out, &with_marker, "", text_w);
                }
                MdLine::Code(code) => {
                    let hl = highlight::highlight_c(code);
                    for l in &hl {
                        if self.color {
                            out.push_str("    ");
                            out.push_str(&highlight::line_to_ansi(l));
                            out.push_str(RESET);
                            out.push('\n');
                        } else {
                            let text: String = l.iter().map(|(_, t)| t.as_str()).collect();
                            out.push_str("    ");
                            out.push_str(&text);
                            out.push('\n');
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_respects_width() {
        let segs = vec![(Seg::Normal, "un deux trois quatre cinq six sept huit".to_string())];
        let lines = wrap_segments(&segs, 12);
        assert!(lines.len() >= 3);
        for l in &lines {
            let width: usize = l.iter().map(|(_, t)| w(t)).sum();
            assert!(width <= 13, "ligne trop large: {width}");
        }
    }
}
