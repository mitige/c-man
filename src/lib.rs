//! c-man — bibliothèque partagée (c-man CLI + epitech-nano).

pub mod compile;
pub mod config;
pub mod content;
pub mod csscheck;
pub mod editor;
pub mod highlight;
pub mod markdown;
pub mod norme;
pub mod progress;
pub mod project;
pub mod quiz;
pub mod exo;
pub mod htmlcheck;
pub mod render;
pub mod serve;

#[cfg(test)]
mod hl_tests {
    #[test]
    fn js_highlighting_has_colors() {
        let segs = crate::highlight::highlight_code("const x = 42;", "js");
        // au moins un segment doit avoir une couleur de fond/surface différente du défaut
        let non_default = segs
            .iter()
            .flatten()
            .any(|(s, _)| !(s.foreground.r == 255 && s.foreground.g == 255 && s.foreground.b == 255));
        assert!(non_default, "la coloration JS doit produire des couleurs");
    }

    #[test]
    fn highlight_supports_many_langs() {
        for ext in ["js", "py", "rs", "html", "css", "sh", "toml"] {
            let segs = crate::highlight::highlight_code("test = 1", ext);
            assert!(!segs.is_empty(), "coloration pour {ext}");
        }
    }
}

#[cfg(test)]
mod hl_color_tests {
    #[test]
    fn syntect_style_has_rgb() {
        let segs = crate::highlight::highlight_code("int main(void) { return 0; }", "c");
        for (style, text) in segs.iter().flatten() {
            eprintln!("{:?} fg=({},{},{})", text, style.foreground.r, style.foreground.g, style.foreground.b);
        }
        let any_colored = segs.iter().flatten().any(|(s, _)| {
            let c = s.foreground;
            !(c.r == 0 && c.g == 0 && c.b == 0) // pas noir pur = couleur réelle
        });
        assert!(any_colored);
    }
}
