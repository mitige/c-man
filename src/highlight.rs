//! Coloration syntaxique C via syntect (thème sombre embarqué).

use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME: LazyLock<Theme> = LazyLock::new(|| {
    let set = ThemeSet::load_defaults();
    set.themes["base16-ocean.dark"].clone()
});

/// Thème de l'éditeur : plus riche en contraste (lisibilité du code,
/// évite les erreurs d'inattention).
static EDITOR_THEME: LazyLock<Theme> = LazyLock::new(|| {
    let set = ThemeSet::load_defaults();
    set.themes["base16-mocha.dark"].clone()
});

fn c_syntax() -> &'static SyntaxReference {
    SYNTAXES
        .find_syntax_by_extension("c")
        .expect("syntaxe C introuvable")
}

/// Syntaxe par extension de fichier (éditeur multi-langage).
fn syntax_for(ext: &str) -> &'static SyntaxReference {
    SYNTAXES
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| c_syntax())
}

/// Couleur de fond du thème (pour harmoniser les blocs de code).
pub fn theme_bg() -> syntect::highlighting::Color {
    THEME
        .settings
        .background
        .unwrap_or(syntect::highlighting::Color {
            r: 20,
            g: 24,
            b: 32,
            a: 255,
        })
}

/// Retourne, par ligne, les segments (style, texte) colorés.
pub fn highlight_c(code: &str) -> Vec<Vec<(Style, String)>> {
    highlight_code(code, "c")
}

/// Coloration par langage (extension du fichier : c, js, py, rs, html, css…).
/// Utilise le thème éditeur (plus riche) pour le code, l'autre pour la doc.
pub fn highlight_code(code: &str, ext: &str) -> Vec<Vec<(Style, String)>> {
    highlight_with_theme(code, ext, &EDITOR_THEME)
}

/// Coloration pour la doc (thème sobre).
pub fn highlight_code_doc(code: &str, ext: &str) -> Vec<Vec<(Style, String)>> {
    highlight_with_theme(code, ext, &THEME)
}

fn highlight_with_theme(code: &str, ext: &str, theme: &Theme) -> Vec<Vec<(Style, String)>> {
    let mut hl = HighlightLines::new(syntax_for(ext), theme);
    code.lines()
        .map(|line| {
            let regions = hl
                .highlight_line(line, &SYNTAXES)
                .unwrap_or_else(|_| vec![(Style::default(), line)]);
            regions
                .into_iter()
                .map(|(style, text)| {
                    // retire l'italique (mal rendu dans beaucoup de terminaux)
                    let style = Style {
                        font_style: style.font_style - FontStyle::ITALIC,
                        ..style
                    };
                    (style, text.to_string())
                })
                .collect()
        })
        .collect()
}

/// Rend une ligne colorée en séquences ANSI 24 bits (fond transparent).
pub fn line_to_ansi(segments: &[(Style, String)]) -> String {
    let refs: Vec<(Style, &str)> = segments
        .iter()
        .map(|(s, t)| (*s, t.as_str()))
        .collect();
    syntect::util::as_24_bit_terminal_escaped(&refs[..], false)
}

