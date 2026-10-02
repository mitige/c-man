//! Validation CSS légère — pour la piscine web.

#[derive(Debug)]
pub struct CssFinding {
    pub line: usize,
    pub msg: String,
}

/// Vérifie une feuille CSS : accolades équilibrées, déclarations sans `;`, etc.
pub fn check(src: &str) -> Vec<CssFinding> {
    let mut findings = Vec::new();
    let mut depth = 0i32;
    let mut in_comment = false;
    let mut in_string: Option<char> = None;

    for (n, raw) in src.lines().enumerate() {
        let ln = n + 1;
        let line = raw.trim();
        let mut chars = line.chars().peekable();
        let mut saw_decl = false;
        while let Some(c) = chars.next() {
            if in_comment {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_comment = false;
                }
                continue;
            }
            if let Some(q) = in_string {
                if c == q {
                    in_string = None;
                }
                continue;
            }
            match c {
                '/' if chars.peek() == Some(&'*') => {
                    chars.next();
                    in_comment = true;
                }
                '"' | '\'' => in_string = Some(c),
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth < 0 {
                        findings.push(CssFinding { line: ln, msg: "accolade } sans { correspondante".into() });
                        depth = 0;
                    }
                }
                // une propriété sans ':' (souvent une valeur orpheline)
                ':' => saw_decl = true,
                _ => {}
            }
        }
        // une ligne qui ressemble à une déclaration mais sans ';' à la fin
        // (sauf si c'est un sélecteur, un { ou }, un commentaire, ou le dernier avant })
        if saw_decl && !line.ends_with(';') && !line.ends_with('{') && !line.ends_with('}')
            && !line.starts_with("/*") && !line.is_empty()
        {
            // probablement une déclaration sans point-virgule
            findings.push(CssFinding {
                line: ln,
                msg: "déclaration sans « ; » à la fin".into(),
            });
        }
    }
    if in_comment {
        findings.push(CssFinding { line: 0, msg: "commentaire /* non fermé".into() });
    }
    if depth != 0 {
        findings.push(CssFinding { line: 0, msg: format!("{depth} accolade(s) {{ non fermée(s)") });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_valide_propre() {
        let ok = "body {\n    color: blue;\n    margin: 0;\n}\n";
        assert!(check(ok).is_empty(), "doit être propre: {:?}", check(ok));
    }

    #[test]
    fn accolade_non_fermee() {
        let bad = "body {\n    color: blue;\n";
        assert!(check(bad).iter().any(|f| f.msg.contains("non fermée")));
    }

    #[test]
    fn decl_sans_point_virgule() {
        let bad = "body {\n    color: blue\n}\n";
        assert!(check(bad).iter().any(|f| f.msg.contains("« ; »")), "{:?}", check(bad));
    }

    #[test]
    fn commentaire_non_ferme() {
        let bad = "/* oups\nbody { color: red; }\n";
        assert!(check(bad).iter().any(|f| f.msg.contains("commentaire")));
    }
}
