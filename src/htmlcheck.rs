//! Validation HTML légère — pour la piscine web.
//!
//! Vérifie la structure : balises fermées, attributs essentiels, structure de base.

#[derive(Debug)]
pub struct HtmlFinding {
    pub line: usize,
    pub msg: String,
}

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
    "source", "track", "wbr",
];

/// Vérifie un document HTML. Retourne les problèmes trouvés.
pub fn check(src: &str) -> Vec<HtmlFinding> {
    let mut findings = Vec::new();
    let mut stack: Vec<(String, usize)> = Vec::new(); // balises ouvertes (tag, ligne)
    let mut has_doctype = false;
    let mut has_lang = false;
    let mut has_charset = false;

    for (n, raw) in src.lines().enumerate() {
        let line = raw.trim();
        let ln = n + 1;
        if line.starts_with("<!DOCTYPE") || line.starts_with("<!doctype") {
            has_doctype = true;
        }
        if line.starts_with("<html") && line.contains("lang=") {
            has_lang = true;
        }
        if line.contains("charset=") {
            has_charset = true;
        }
        // img sans alt
        if line.contains("<img") && !line.contains("alt=") {
            findings.push(HtmlFinding {
                line: ln,
                msg: "<img> sans attribut alt (accessibilité)".into(),
            });
        }
        // balises : ouverture / fermeture
        let mut rest = line;
        while let Some(start) = rest.find('<') {
            let Some(end) = rest[start..].find('>') else { break };
            let tag = &rest[start + 1..start + end];
            rest = &rest[start + end + 1..];
            if tag.starts_with("!") || tag.starts_with("?") {
                continue; // doctype, commentaire
            }
            let closing = tag.starts_with('/');
            let name: String = tag
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_lowercase();
            if name.is_empty() || VOID.contains(&name.as_str()) {
                continue;
            }
            let self_closing = tag.ends_with('/');
            if closing {
                // ferme la balise correspondante la plus récente
                if let Some(pos) = stack.iter().rposition(|(t, _)| t == &name) {
                    // tout ce qui est au-dessus n'est pas fermé → mal imbriqué
                    for (t, l) in stack.drain(pos + 1..) {
                        findings.push(HtmlFinding {
                            line: l,
                            msg: format!("<{t}> ouverte ligne {l} mais jamais fermée (avant </{name}>)"),
                        });
                    }
                    stack.pop();
                } else {
                    findings.push(HtmlFinding {
                        line: ln,
                        msg: format!("</{name}> fermée sans être ouverte"),
                    });
                }
            } else if !self_closing {
                stack.push((name, ln));
            }
        }
    }
    // balises restées ouvertes
    for (t, l) in stack.drain(..) {
        findings.push(HtmlFinding {
            line: l,
            msg: format!("<{t}> ouverte mais jamais fermée"),
        });
    }
    // structure de base
    if !has_doctype {
        findings.push(HtmlFinding { line: 1, msg: "<!DOCTYPE html> manquant en première ligne".into() });
    }
    if !has_lang {
        findings.push(HtmlFinding { line: 0, msg: "<html> sans attribut lang (accessibilité)".into() });
    }
    if !has_charset {
        findings.push(HtmlFinding { line: 0, msg: "charset non déclaré (<meta charset=\"UTF-8\">)".into() });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_valide_propre() {
        let ok = "<!DOCTYPE html>\n<html lang=\"fr\">\n<head>\n<meta charset=\"UTF-8\">\n<title>t</title>\n</head>\n<body>\n<p>hi</p>\n<img src=\"a.png\" alt=\"a\">\n</body>\n</html>\n";
        assert!(check(ok).is_empty(), "doit être propre: {:?}", check(ok));
    }

    #[test]
    fn balise_non_fermee() {
        let bad = "<!DOCTYPE html>\n<html lang=\"fr\">\n<head><meta charset=\"UTF-8\"></head>\n<body>\n<p>oups\n</body>\n</html>\n";
        let f = check(bad);
        assert!(f.iter().any(|x| x.msg.contains("<p>")), "p non fermée détectée: {f:?}");
    }

    #[test]
    fn img_sans_alt() {
        let bad = "<!DOCTYPE html>\n<html lang=\"fr\">\n<head><meta charset=\"UTF-8\"></head>\n<body><img src=\"x.png\"></body></html>\n";
        assert!(check(bad).iter().any(|x| x.msg.contains("alt")));
    }

    #[test]
    fn pas_de_doctype() {
        let bad = "<html lang=\"fr\"><head><meta charset=\"UTF-8\"></head><body></body></html>\n";
        assert!(check(bad).iter().any(|x| x.msg.contains("DOCTYPE")));
    }
}
