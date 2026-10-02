//! Compilation intégrée : lance gcc/make, capture et explique les diagnostics,
//! pointe vers la fiche c-man pertinente.

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub level: DiagLevel,
    pub message: String,
    /// Fiche c-man recommandée (id).
    pub hint_fiche: Option<&'static str>,
    /// Explication courte en français.
    pub hint_text: Option<&'static str>,
}

#[derive(Debug)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub raw: String,
}

/// Parse la sortie gcc/clang : `fichier:ligne:col: error|warning|note: message`.
pub fn parse_diagnostics(output: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for line in output.lines() {
        if let Some(d) = parse_diag_line(line) {
            diags.push(d);
        }
    }
    diags
}

fn parse_diag_line(line: &str) -> Option<Diagnostic> {
    // cas linker : main.c:(.text+0x1f): undefined reference to `sqrt'
    if line.contains(": undefined reference") || line.contains(": multiple definition") {
        let file = line.split(':').next()?.trim();
        if file.ends_with(".c") || file.ends_with(".h") || file.ends_with(".o") {
            let message = line.splitn(2, ": ").nth(1)?.trim();
            let (hint_fiche, hint_text) = explain_diagnostic(message);
            return Some(Diagnostic {
                file: file.to_string(),
                line: 0,
                col: 0,
                level: DiagLevel::Error,
                message: message.to_string(),
                hint_fiche,
                hint_text,
            });
        }
    }
    // forme: path/file.c:12:5: error: message  (ou "warning:"/"note:")
    //       ou path/file.c:12: error: message (sans colonne)
    let bytes = line;
    let mut parts = bytes.splitn(4, ':');
    let file = parts.next()?.trim();
    if file.is_empty() || !(file.ends_with(".c") || file.ends_with(".h")) {
        return None;
    }
    let line_no: usize = parts.next()?.trim().parse().ok()?;
    let rest = parts.next()?;
    let (col, rest) = match rest.trim().parse::<usize>() {
        Ok(c) => (c, parts.next()?),
        Err(_) => (0, rest),
    };
    let (level, message) = if let Some(m) = rest.splitn(2, "error:").nth(1) {
        (DiagLevel::Error, m.trim())
    } else if let Some(m) = rest.splitn(2, "warning:").nth(1) {
        (DiagLevel::Warning, m.trim())
    } else if let Some(m) = rest.splitn(2, "note:").nth(1) {
        (DiagLevel::Note, m.trim())
    } else {
        return None;
    };
    let (hint_fiche, hint_text) = explain_diagnostic(message);
    Some(Diagnostic {
        file: file.to_string(),
        line: line_no,
        col,
        level,
        message: message.to_string(),
        hint_fiche,
        hint_text,
    })
}

/// Associe un message gcc à une explication + fiche c-man.
fn explain_diagnostic(msg: &str) -> (Option<&'static str>, Option<&'static str>) {
    let m = msg.to_lowercase();
    let rule: &[(&[&str], &str, &str)] = &[
        (&["implicit declaration"], "headers-include-guards",
         "Fonction utilisée sans prototype : ajoute le bon #include. Sans déclaration, C devine les types — et devine mal."),
        (&["undefined reference"], "linking",
         "L'éditeur de liens ne trouve pas le symbole : fichier .c oublié, bibliothèque manquante (-lm ?), ou -l placé avant les .o."),
        (&["incompatible pointer"], "pointeurs",
         "Le type pointé ne correspond pas. Relis la signature : niveau d'indirection (*) et type de base doivent correspondre."),
        (&["format '%'"], "printf",
         "Le format printf ne correspond pas au type de l'argument (%d attend un int, %zu un size_t...)."),
        (&["unused variable", "unused parameter"], "flags-compilation",
         "Variable/paramètre inutilisé : supprime-le ou utilise-le. Avec -Werror, c'est bloquant."),
        (&["may be used uninitialized", "uninitialized"], "portee-variables",
         "Variable potentiellement lue avant initialisation : initialise toujours à la déclaration."),
        (&["comparison of integer expressions of different signedness", "signedness"], "debordements-entiers",
         "Comparaison signé/non signé : le signé est converti en non signé (une valeur négative devient énorme). Caste explicitement."),
        (&["returning", "from a function"], "stack-vs-heap",
         "Tu retournes probablement l'adresse d'une variable locale : sa mémoire meurt au return."),
        (&["lvalue required"], "operateurs",
         "Le membre de gauche d'une affectation doit être modifiable (variable, *p, t[i]...)."),
        (&["expected ';'"], "variables-types",
         "Point-virgule manquant (souvent à la ligne PRÉCÉDENTE) ou structure sans ';' final."),
        (&["expected declaration or statement"], "fonctions",
         "Erreur de structure : accolade déséquilibrée ou instruction hors fonction. Vérifie tes {}."),
        (&["duplicate member", "redefinition"], "structures",
         "Nom déjà utilisé : membre en double dans la structure, ou fonction définie deux fois."),
        (&["dereferencing pointer to incomplete type"], "structures",
         "La structure n'est pas définie ici : il manque sa définition (header inclus ?)."),
        (&["too few arguments", "too many arguments"], "fonctions",
         "Le nombre d'arguments ne correspond pas au prototype. Relis la signature dans le .h."),
        (&["conflicting types"], "headers-include-guards",
         "Deux déclarations différentes pour la même fonction : le prototype du .h et la définition du .c divergent."),
        (&["array subscript is not an integer", "subscripted value"], "tableaux",
         "Tu indexes quelque chose qui n'est pas un tableau/pointeur, ou avec un mauvais type."),
        (&["assignment makes pointer from integer", "cast to pointer from integer"], "pointeurs",
         "Tu convertis un entier en pointeur sans cast : presque toujours un oubli de prototype ou de &."),
        (&["cast from pointer to integer", "makes integer from pointer"], "pointeurs",
         "Conversion pointeur↔entier dangereuse : vérifie que tu ne confonds pas adresse et valeur."),
        (&["iso c99", "c99"], "flags-compilation",
         "Tu utilises une extension au-delà de C99 : ajoute le bon #define de feature test ou reste en C99 strict."),
        (&["multiple definition"], "linking",
         "Symbole défini dans plusieurs .c : une seule définition autorisée (les autres en extern)."),
        (&["discards 'const' qualifier", "discards const"], "const",
         "Tu retires un const : le pointeur résultat permettrait de modifier une donnée protégée."),
        (&["division by zero"], "operateurs",
         "Division par zéro détectée à la compilation — vérifie le diviseur."),
        (&["no such file or directory"], "gcc",
         "Fichier introuvable : chemin du #include incorrect, ou option -I manquante."),
    ];
    for (needles, fiche, text) in rule {
        if needles.iter().any(|n| m.contains(n)) {
            return (Some(fiche), Some(text));
        }
    }
    (None, None)
}

/// Compile un fichier ou lance make dans un répertoire.
pub fn compile_target(target: &Path) -> CompileResult {
    let (cmd, args, cwd): (String, Vec<String>, Option<&Path>) = if target.is_dir()
        && (target.join("Makefile").exists() || target.join("makefile").exists())
    {
        ("make".into(), vec!["re".into()], Some(target))
    } else if target.is_dir() {
        // pas de Makefile : compile tous les .c du dossier
        let out = "/tmp/c-man-build.out".to_string();
        let mut args = vec![
            "-Wall".into(), "-Wextra".into(), "-Werror".into(), "-std=c99".into(), "-g".into(),
            "-o".into(), out,
        ];
        if let Ok(rd) = std::fs::read_dir(target) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "c") {
                    // le cwd est déjà le dossier → juste le nom du fichier
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        args.push(name.to_string());
                    }
                }
            }
        }
        ("gcc".into(), args, Some(target))
    } else {
        let out = "/tmp/c-man-build.out".to_string();
        (
            "gcc".into(),
            vec![
                "-Wall".into(),
                "-Wextra".into(),
                "-Werror".into(),
                "-std=c99".into(),
                "-g".into(),
                "-o".into(),
                out,
                target.to_string_lossy().into(),
            ],
            None,
        )
    };
    let result = Command::new(&cmd)
        .args(&args)
        .current_dir(cwd.unwrap_or_else(|| Path::new(".")))
        .output();
    match result {
        Ok(out) => {
            let raw = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let diagnostics = parse_diagnostics(&raw);
            CompileResult {
                success: out.status.success(),
                diagnostics,
                raw,
            }
        }
        Err(e) => CompileResult {
            success: false,
            diagnostics: vec![],
            raw: format!("impossible de lancer {cmd}: {e}"),
        },
    }
}

/// Lance valgrind sur un binaire et retourne le rapport (texte).
pub fn run_valgrind(binary: &Path) -> CompileResult {
    let args = [
        "--leak-check=full",
        "--show-leak-kinds=all",
        "--track-origins=yes",
        "--error-exitcode=42",
    ];
    let result = Command::new("valgrind").args(args).arg(binary).output();
    match result {
        Ok(out) => {
            let raw = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            CompileResult {
                success: out.status.success(),
                diagnostics: vec![],
                raw,
            }
        }
        Err(e) => CompileResult {
            success: false,
            diagnostics: vec![],
            raw: format!(
                "valgrind introuvable ou erreur : {e}\n(installe-le : sudo pacman -S valgrind)"
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_line() {
        let out = "main.c:12:5: error: implicit declaration of function 'my_putstr'\nmain.c:13:1: warning: unused variable 'x'\n";
        let diags = parse_diagnostics(out);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].level, DiagLevel::Error);
        assert_eq!(diags[0].line, 12);
        assert_eq!(diags[0].col, 5);
        assert_eq!(diags[0].hint_fiche, Some("headers-include-guards"));
        assert_eq!(diags[1].level, DiagLevel::Warning);
    }

    #[test]
    fn parse_linker_error() {
        let out = "/usr/bin/ld: /tmp/cc1.o: in function `main':\nmain.c:(.text+0x1f): undefined reference to `sqrt'\n";
        let diags = parse_diagnostics(out);
        assert!(diags.iter().any(|d| d.hint_fiche == Some("linking")));
    }

    #[test]
    fn noise_ignored() {
        let diags = parse_diagnostics("gcc: fatal error: cannot execute\ncompilation terminated.");
        assert!(diags.is_empty());
    }
}
