//! Vérificateur de la Coding Style Epitech (style « epitech-clang »/banana).
//!
//! Heuristique ligne à ligne avec neutralisation des chaînes/commentaires :
//! ce n'est pas un parseur C complet, mais il couvre les règles classiques de
//! la piscine avec fichier, ligne, colonne, explication et auto-fix quand
//! c'est sûr.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// ------------------------------------------------------------------ types

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Minor,
    Major,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Minor => write!(f, "MINOR"),
            Severity::Major => write!(f, "MAJOR"),
        }
    }
}

/// Correction automatique applicable à une ligne (index 0-based).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fix {
    /// Remplace tout le contenu de la ligne.
    ReplaceLine { line: usize, content: String },
    /// Insère une ligne avant `line`.
    InsertBefore { line: usize, content: String },
    /// Supprime la ligne.
    DeleteLine { line: usize },
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub line: usize, // 1-based
    pub col: usize,  // 1-based
    pub severity: Severity,
    pub rule: &'static str,
    pub message: String,
    pub fix: Option<Fix>,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: [{}] {} ({})",
            self.line, self.col, self.severity, self.message, self.rule
        )
    }
}

#[derive(Debug, Clone)]
pub struct FileReport {
    pub path: PathBuf,
    pub findings: Vec<Finding>,
}

impl FileReport {
    pub fn count(&self, sev: Severity) -> usize {
        self.findings.iter().filter(|f| f.severity == sev).count()
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub files: Vec<FileReport>,
}

impl Report {
    pub fn total(&self, sev: Severity) -> usize {
        self.files.iter().map(|f| f.count(sev)).sum()
    }
    pub fn findings_count(&self) -> usize {
        self.files.iter().map(|f| f.findings.len()).sum()
    }
    pub fn fixable_count(&self) -> usize {
        self.files
            .iter()
            .flat_map(|f| &f.findings)
            .filter(|f| f.fix.is_some())
            .count()
    }
}

// ------------------------------------------------------------------ config

#[derive(Debug, Clone)]
pub struct NormeConfig {
    pub max_columns: usize,
    pub max_function_lines: usize,
    pub max_functions_per_file: usize,
    pub forbid_for: bool,
    pub forbid_ternary: bool,
    pub forbid_switch: bool,
    pub forbid_goto: bool,
    pub return_parens: bool,
    pub comments_in_function: bool,
}

impl Default for NormeConfig {
    fn default() -> Self {
        Self {
            max_columns: 80,
            max_function_lines: 25,
            max_functions_per_file: 5,
            forbid_for: true,
            forbid_ternary: true,
            forbid_switch: false,
            forbid_goto: true,
            return_parens: true,
            comments_in_function: false,
        }
    }
}

// ------------------------------------------------------------------ explications

/// (titre, pourquoi, comment corriger)
pub fn explain_rule(rule: &str) -> (&'static str, &'static str, &'static str) {
    match rule {
        "N-COL80" => (
            "Ligne de plus de 80 colonnes",
            "La Norme impose 80 colonnes : lisibilité, revue de code côte à côte, \
             et la norminette te sanctionne direct.",
            "Découpe l'expression : variable intermédiaire, découpage de la déclaration, \
             ou fonction auxiliaire.",
        ),
        "N-FUNC25" => (
            "Fonction de plus de 25 lignes",
            "Une fonction = une tâche. Au-delà de 25 lignes (accolades et lignes vides \
             exclues du décompte selon les versions), elle fait trop de choses.",
            "Extrais des sous-étapes dans des fonctions auxiliaires (statique si \
             interne). Le découpage clarifie presque toujours l'algo.",
        ),
        "N-FUNCCOUNT" => (
            "Plus de 5 fonctions dans le fichier",
            "La Norme limite à 5 fonctions par fichier .c pour forcer une organisation \
             en modules cohérents.",
            "Regroupe les fonctions par responsabilité dans plusieurs fichiers .c \
             (ex: list_add.c, list_remove.c).",
        ),
        "N-FOR" => (
            "Boucle for interdite",
            "En piscine, la Norme interdit for : tout se fait en while. C'est \
             contraignant mais ça t'oblige à maîtriser l'initialisation, la condition \
             et l'incrément séparément.",
            "Réécris en while : init avant, condition dans le while, incrément en fin \
             de corps.",
        ),
        "N-TERNARY" => (
            "Opérateur ternaire interdit",
            "Les ternaires sont interdits en piscine : ils cachent des branches et \
             poussent à écrire des expressions trop denses.",
            "Remplace par un if/else explicite.",
        ),
        "N-GOTO" => (
            "goto interdit",
            "goto rend le flot d'exécution illisible et est interdit par la Norme.",
            "Restructure : fonction auxiliaire avec return, ou booléen de contrôle.",
        ),
        "N-SWITCH" => (
            "switch à éviter",
            "Selon les sujets de piscine, switch est interdit. Vérifie ton sujet.",
            "Une cascade if/else fait le même travail.",
        ),
        "N-HEADER" => (
            "En-tête de fichier manquant ou invalide",
            "Chaque fichier doit commencer par l'en-tête Epitech (Made by, Login, \
             dates). Son absence est une faute MAJOR.",
            "Insère le bloc d'en-tête normalisé en tête de fichier (c-man fix le fait).",
        ),
        "N-GLOBAL" => (
            "Variable globale",
            "Les variables globales sont interdites en piscine : elles rendent le code \
             impossible à tester et à raisonner.",
            "Passe l'état en paramètre, ou encapsule dans une structure passée aux \
             fonctions.",
        ),
        "N-MULTISTMT" => (
            "Plusieurs instructions sur une ligne",
            "Une seule instruction par ligne : le débogage ligne à ligne et la \
             relecture l'exigent.",
            "Mets chaque instruction sur sa propre ligne.",
        ),
        "N-MULTIDECL" => (
            "Déclarations multiples sur une ligne",
            "int a, b; est interdit : chaque variable a droit à sa ligne (et à son \
             initialisation explicite).",
            "Une déclaration par ligne.",
        ),
        "N-TRAILSPACE" => (
            "Espaces en fin de ligne",
            "Les espaces invisibles en fin de ligne polluent les diffs et sont \
             sanctionnés par la norminette.",
            "Supprime-les (c-man fix le fait).",
        ),
        "N-TAB" => (
            "Tabulation dans l'indentation",
            "L'indentation Epitech se fait en espaces (4 par niveau), pas en \
             tabulations.",
            "Remplace les tabs par 4 espaces (c-man fix le fait).",
        ),
        "N-KEYWORDSPACE" => (
            "Espace manquant après un mot-clé",
            "if(, while(, return( ... : un espace sépare le mot-clé de la parenthèse.",
            "Ajoute l'espace (c-man fix le fait).",
        ),
        "N-BLANKLINES" => (
            "Lignes vides consécutives",
            "Une seule ligne vides de séparation ; jamais deux d'affilée dans du code.",
            "Supprime les lignes vides en trop (c-man fix le fait).",
        ),
        "N-BRACEBLANK" => (
            "Ligne vide après { ou avant }",
            "Pas de ligne vide juste après l'accolade ouvrante ni juste avant la \
             fermante d'une fonction.",
            "Supprime cette ligne vide (c-man fix le fait).",
        ),
        "N-FUNCBRACE" => (
            "Accolade de fonction sur la ligne de la signature",
            "L'accolade ouvrante d'une fonction se place seule sur la ligne suivante \
             (style Epitech).",
            "Descends l'accolade sur sa propre ligne (c-man fix le fait).",
        ),
        "N-RETURNPAREN" => (
            "return sans parenthèses",
            "Certaines versions de la Norme exigent return (valeur); avec parenthèses.",
            "Entoure la valeur de parenthèses (c-man fix le fait).",
        ),
        "N-SNAKECASE" => (
            "Nom de fonction non snake_case",
            "Les fonctions se nomment en minuscules avec underscores (my_putstr).",
            "Renomme en snake_case.",
        ),
        "N-COMMASPACE" => (
            "Espace manquant après une virgule",
            "f(a, b) : un espace suit chaque virgule.",
            "Ajoute l'espace (c-man fix le fait).",
        ),
        "N-STARSTYLE" => (
            "Style de pointeur (int* p)",
            "Style Epitech : l'étoile se colle au nom — int *p — pas au type.",
            "Déplace l'étoile (c-man fix le fait).",
        ),
        "N-VOIDPARAM" => (
            "Liste de paramètres vide sans void",
            "int f() ne déclare pas une fonction sans paramètre en C : écris \
             int f(void).",
            "Ajoute void (c-man fix le fait).",
        ),
        "N-FUNCSEP" => (
            "Séparation entre fonctions",
            "Une (et une seule) ligne vide sépare deux définitions de fonctions.",
            "Ajuste les lignes vides (c-man fix le fait).",
        ),
        "N-INCLUDEGUARD" => (
            "Include guard manquant dans le .h",
            "Sans guard, une double inclusion casse la compilation. Toujours \
             #ifndef/#define/#endif.",
            "Ajoute la garde (c-man fix insère le squelette).",
        ),
        "N-COMMENTFUNC" => (
            "Commentaire dans le corps d'une fonction",
            "Les versions strictes de la Norme interdisent les commentaires dans les \
             fonctions : si le code a besoin d'un commentaire, il doit être simplifié.",
            "Déplace le commentaire au-dessus de la fonction, ou simplifie le code.",
        ),
        "C-A3" => (
            "Retour à la ligne en fin de fichier",
            "La norme l'exige : tout fichier doit se terminer par un '\\n'. \
             Sans lui, certains outils (gcc, diff, la moulinette) tronquent ou \
             mélangent la dernière ligne.",
            "Ajoute une ligne vide finale (c-man fix le fait automatiquement).",
        ),
        _ => ("Règle de style", "Voir la documentation Epitech de ta promo.", "—"),
    }
}

/// Code officiel banana/norminette correspondant à ma règle interne.
/// Permet de croiser la sortie de la moulinette avec c-man.
pub fn official_code(rule: &str) -> Option<&'static str> {
    Some(match rule {
        "N-COL80" => "C-F3",
        "N-FUNC25" => "C-F4",
        "N-FUNCCOUNT" => "C-O3",
        "N-GOTO" => "C-C3",
        "N-HEADER" => "C-G1",
        "N-GLOBAL" => "C-G4",
        "N-MULTISTMT" => "C-L1",
        "N-MULTIDECL" => "C-L5",
        "N-TRAILSPACE" => "C-G7",
        "N-TAB" => "C-L2",
        "N-KEYWORDSPACE" | "N-COMMASPACE" => "C-L3",
        "N-BLANKLINES" | "N-BRACEBLANK" => "C-L6",
        "N-FUNCBRACE" => "C-L4",
        "N-SNAKECASE" => "C-F2",
        "N-STARSTYLE" => "C-V3",
        "N-VOIDPARAM" => "C-F6",
        "N-FUNCSEP" => "C-G2",
        "N-INCLUDEGUARD" => "C-H2",
        "N-COMMENTFUNC" => "C-F8",
        _ => return None,
    })
}

/// Traduit un code officiel (C-A3, C-F3...) en ma règle interne.
pub fn official_to_internal(code: &str) -> Option<&'static str> {
    if code == "C-A3" {
        return Some("C-A3");
    }
    // inverse la table official_code
    for internal in [
        "N-COL80", "N-FUNC25", "N-FUNCCOUNT", "N-GOTO", "N-HEADER", "N-GLOBAL",
        "N-MULTISTMT", "N-MULTIDECL", "N-TRAILSPACE", "N-TAB", "N-KEYWORDSPACE",
        "N-BLANKLINES", "N-FUNCBRACE", "N-SNAKECASE", "N-STARSTYLE", "N-VOIDPARAM",
        "N-FUNCSEP", "N-INCLUDEGUARD", "N-COMMENTFUNC",
    ] {
        if official_code(internal) == Some(code) {
            return Some(internal);
        }
    }
    None
}

// ------------------------------------------------------------------ sanitizer

/// Neutralise chaînes, caractères et commentaires : retourne une version par
/// ligne où leur contenu est remplacé par des espaces (positions préservées).
fn sanitize(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block_comment = false;
    for line in src.lines() {
        let bytes: Vec<char> = line.chars().collect();
        let mut clean: Vec<char> = bytes.clone();
        let mut i = 0;
        let mut in_str = false;
        let mut in_chr = false;
        while i < bytes.len() {
            let c = bytes[i];
            if in_block_comment {
                clean[i] = ' ';
                if c == '*' && i + 1 < bytes.len() && bytes[i + 1] == '/' {
                    clean[i + 1] = ' ';
                    i += 2;
                    in_block_comment = false;
                    continue;
                }
            } else if in_str {
                if c == '\\' && i + 1 < bytes.len() {
                    clean[i] = ' ';
                    clean[i + 1] = ' ';
                    i += 2;
                    continue;
                }
                clean[i] = ' ';
                if c == '"' {
                    in_str = false;
                }
            } else if in_chr {
                if c == '\\' && i + 1 < bytes.len() {
                    clean[i] = ' ';
                    clean[i + 1] = ' ';
                    i += 2;
                    continue;
                }
                clean[i] = ' ';
                if c == '\'' {
                    in_chr = false;
                }
            } else if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == '/' {
                for j in i..bytes.len() {
                    clean[j] = ' ';
                }
                break;
            } else if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == '*' {
                clean[i] = ' ';
                clean[i + 1] = ' ';
                in_block_comment = true;
                i += 2;
                continue;
            } else if c == '"' {
                clean[i] = ' ';
                in_str = true;
            } else if c == '\'' {
                clean[i] = ' ';
                in_chr = true;
            }
            i += 1;
        }
        out.push(clean.into_iter().collect());
    }
    out
}

// ------------------------------------------------------------------ fonctions C

/// Une définition de fonction détectée.
#[derive(Debug)]
struct FuncDef {
    name: String,
    start: usize,   // ligne de la signature (0-based)
    open: usize,    // ligne de l'accolade ouvrante
    close: usize,   // ligne de l'accolade fermante
    body_lines: usize,
}

fn is_type_line(s: &str) -> bool {
    let first = s.split_whitespace().next().unwrap_or("");
    matches!(
        first,
        "int" | "char" | "void" | "long" | "short" | "float" | "double" | "unsigned"
            | "signed" | "size_t" | "ssize_t" | "struct" | "static" | "const" | "bool"
            | "FILE" | "enum"
    ) || first.starts_with("t_")
}

/// Détecte les définitions de fonctions (signature à col 0, accolade à col 0).
fn find_functions(clean: &[String]) -> Vec<FuncDef> {
    let mut funcs = Vec::new();
    let mut i = 0;
    while i < clean.len() {
        let line = &clean[i];
        // candidate : ligne col 0 non préprocesseur contenant ( et finissant par )
        // (signature possiblement multi-lignes)
        if !line.starts_with(|c: char| c.is_whitespace())
            && !line.trim_start().starts_with('#')
            && line.contains('(')
            && is_type_line(line)
        {
            // reconstruit la signature complète (jusqu'à ')')
            let mut sig = line.clone();
            let mut j = i;
            let mut paren: i32 = sig.matches('(').count() as i32
                - sig.matches(')').count() as i32;
            while paren > 0 && j + 1 < clean.len() {
                j += 1;
                sig.push(' ');
                sig.push_str(clean[j].trim());
                paren += clean[j].matches('(').count() as i32
                    - clean[j].matches(')').count() as i32;
            }
            // prochaine ligne non vide
            let mut k = j + 1;
            while k < clean.len() && clean[k].trim().is_empty() {
                k += 1;
            }
            let brace_on_sig = sig.trim_end().ends_with('{');
            let brace_next = k < clean.len() && clean[k].trim() == "{";
            if paren == 0 && sig.contains(')') && (brace_on_sig || brace_next) {
                // nom de la fonction : ident avant la première (
                let name = sig[..sig.find('(').unwrap()]
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .trim_start_matches('*')
                    .to_string();
                let open = if brace_on_sig { i } else { k };
                // trouve la fermante : compte les accolades
                let mut depth = 0i32;
                let mut close = open;
                for (idx, l) in clean.iter().enumerate().skip(open) {
                    depth += l.matches('{').count() as i32;
                    depth -= l.matches('}').count() as i32;
                    if depth == 0 && idx >= open {
                        close = idx;
                        break;
                    }
                }
                // lignes de corps : entre { et } exclues, hors lignes vides
                // (garde : corps vide ou détection bancale → 0, jamais de slice inversé)
                let body_lines = if close > open {
                    clean[open + 1..close.min(clean.len())]
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .count()
                } else {
                    0
                };
                funcs.push(FuncDef {
                    name,
                    start: i,
                    open,
                    close,
                    body_lines,
                });
                i = close + 1;
                continue;
            }
        }
        i += 1;
    }
    funcs
}

// ------------------------------------------------------------------ règles

/// Normalise une ligne : tabs d'indentation → 4 espaces, trailing trimmé.
/// Utilisé par toutes les fixes de ligne pour que le choix de l'une n'importe pas.
fn normalize_line(raw: &str) -> String {
    let tabs = raw.chars().take_while(|c| *c == '\t').count();
    format!("{}{}", "    ".repeat(tabs), raw.trim_start_matches('\t').trim_end())
}

fn add(findings: &mut Vec<Finding>, line: usize, col: usize, sev: Severity, rule: &'static str, msg: impl Into<String>, fix: Option<Fix>) {
    findings.push(Finding {
        line,
        col,
        severity: sev,
        rule,
        message: msg.into(),
        fix,
    });
}

/// Vérifie un fichier source (.c ou .h). Retourne les findings triés.
pub fn check_source(path: &Path, src: &str, cfg: &NormeConfig) -> Vec<Finding> {
    let mut findings = Vec::new();
    let raw_lines: Vec<&str> = src.lines().collect();
    let clean = sanitize(src);
    let is_header = path.extension().is_some_and(|e| e == "h");

    // ---- règles ligne à ligne
    let mut blank_run = 0usize;
    for (i, raw) in raw_lines.iter().enumerate() {
        let n = i + 1;
        let code = &clean[i];

        // colonnes
        let width = raw.chars().count();
        if width > cfg.max_columns {
            add(&mut findings, n, cfg.max_columns + 1, Severity::Major, "N-COL80",
                format!("ligne de {width} colonnes (max {})", cfg.max_columns), None);
        }
        // trailing whitespace
        if raw.ends_with([' ', '\t']) {
            add(&mut findings, n, raw.trim_end().len() + 1, Severity::Minor, "N-TRAILSPACE",
                "espaces/tabulations en fin de ligne", Some(Fix::ReplaceLine {
                    line: i,
                    content: normalize_line(raw),
                }));
        }
        // tab indentation
        if raw.starts_with('\t') {
            add(&mut findings, n, 1, Severity::Minor, "N-TAB",
                "indentation par tabulation", Some(Fix::ReplaceLine { line: i, content: normalize_line(raw) }));
        }
        // lignes vides consécutives
        if raw.trim().is_empty() {
            blank_run += 1;
            if blank_run >= 2 {
                add(&mut findings, n, 1, Severity::Minor, "N-BLANKLINES",
                    "lignes vides consécutives", Some(Fix::DeleteLine { line: i }));
            }
        } else {
            blank_run = 0;
        }

        // mots-clés collés à la parenthèse : if( while( return( switch(
        for kw in ["if", "while", "return", "switch"] {
            let pat = format!("{kw}(");
            if let Some(pos) = find_word_pattern(code, kw, &pat) {
                add(&mut findings, n, pos + kw.len() + 1, Severity::Minor, "N-KEYWORDSPACE",
                    format!("espace manquant après « {kw} »"), Some(Fix::ReplaceLine {
                        line: i,
                        content: replace_word_pattern(raw, kw, &format!("{kw} (")),
                    }));
            }
        }
        // virgule non suivie d'espace
        let mut search_from = 0;
        while let Some(rel) = code[search_from..].find(',') {
            let pos = search_from + rel;
            let after = code[pos + 1..].chars().next();
            if let Some(a) = after {
                if a != ' ' && a != ')' && a != '\n' && a != ']' {
                    add(&mut findings, n, pos + 2, Severity::Minor, "N-COMMASPACE",
                        "espace manquant après la virgule", Some(Fix::ReplaceLine {
                            line: i,
                            content: format!("{} {}", &raw[..pos + 1], &raw[pos + 1..]),
                        }));
                }
            }
            search_from = pos + 1;
        }

        // structures de contrôle interdites
        if cfg.forbid_for && find_keyword(code, "for") {
            let col = code.find("for").map(|p| p + 1).unwrap_or(1);
            add(&mut findings, n, col, Severity::Major, "N-FOR",
                "boucle for interdite (piscine : while uniquement)", None);
        }
        if cfg.forbid_goto && find_keyword(code, "goto") {
            add(&mut findings, n, 1, Severity::Major, "N-GOTO", "goto interdit", None);
        }
        if cfg.forbid_switch && find_keyword(code, "switch") {
            add(&mut findings, n, 1, Severity::Major, "N-SWITCH", "switch interdit par ce sujet", None);
        }
        if cfg.forbid_ternary && has_ternary(code) {
            add(&mut findings, n, 1, Severity::Minor, "N-TERNARY",
                "opérateur ternaire interdit en piscine", None);
        }
        // plusieurs instructions par ligne : "; code" après le premier ;
        // (sauf ligne de continuation de macro, qui finit par « \ »)
        if !code.trim_end().ends_with('\\') {
            if let Some(pos) = multi_statement_pos(code) {
                add(&mut findings, n, pos + 1, Severity::Major, "N-MULTISTMT",
                    "plusieurs instructions sur la même ligne", None);
            }
        }
        // return sans parenthèses
        if cfg.return_parens {
            let t = code.trim();
            if let Some(rest) = t.strip_prefix("return ") {
                let rest = rest.trim();
                if !rest.starts_with('(') && rest != ";" {
                    add(&mut findings, n, raw.len() - rest.len(), Severity::Minor, "N-RETURNPAREN",
                        "return sans parenthèses", Some(Fix::ReplaceLine {
                            line: i,
                            content: raw.replacen("return ", "return (", 1)
                                .trim_end().trim_end_matches(';').to_string() + ");",
                        }));
                }
            }
        }
        // style pointeur : type* name
        if let Some(pos) = pointer_style_pos(code) {
            add(&mut findings, n, pos + 1, Severity::Minor, "N-STARSTYLE",
                "style Epitech : coller l'étoile au nom (int *p, pas int* p)",
                Some(fix_pointer_style(raw, i)));
        }
    }

    // ---- en-tête de fichier (sauf .h ? si, les deux)
    check_header(&raw_lines, &mut findings);

    // ---- fonctions
    let funcs = find_functions(&clean);
    for f in &funcs {
        if f.body_lines > cfg.max_function_lines {
            add(&mut findings, f.start + 1, 1, Severity::Major, "N-FUNC25",
                format!("fonction « {} » : {} lignes de corps (max {})",
                        f.name, f.body_lines, cfg.max_function_lines), None);
        }
        if f.name.chars().any(|c| c.is_uppercase()) {
            add(&mut findings, f.start + 1, 1, Severity::Minor, "N-SNAKECASE",
                format!("fonction « {} » : nom attendu en snake_case", f.name), None);
        }
        // accolade de fonction sur la ligne de signature
        if f.open == f.start && clean[f.start].trim_end().ends_with('{') {
            let sig = raw_lines[f.start].trim_end().trim_end_matches('{').trim_end();
            add(&mut findings, f.start + 1, raw_lines[f.start].len(), Severity::Minor, "N-FUNCBRACE",
                "accolade ouvrante de fonction sur la ligne de la signature",
                Some(Fix::ReplaceLine { line: f.start, content: format!("{sig}\n{{") }));
        }
        // void dans les paramètres vides
        if let Some(p) = clean[f.start].find("()") {
            add(&mut findings, f.start + 1, p + 1, Severity::Minor, "N-VOIDPARAM",
                format!("« {}() » devrait être « {}(void) »", f.name, f.name),
                Some(Fix::ReplaceLine {
                    line: f.start,
                    content: raw_lines[f.start].replacen("()", "(void)", 1),
                }));
        }
        // lignes vides après { / avant }
        if f.open + 1 < f.close && raw_lines[f.open + 1].trim().is_empty() {
            add(&mut findings, f.open + 2, 1, Severity::Minor, "N-BRACEBLANK",
                "ligne vide après l'accolade ouvrante",
                Some(Fix::DeleteLine { line: f.open + 1 }));
        }
        if f.close > f.open + 1 && raw_lines[f.close - 1].trim().is_empty() {
            add(&mut findings, f.close, 1, Severity::Minor, "N-BRACEBLANK",
                "ligne vide avant l'accolade fermante",
                Some(Fix::DeleteLine { line: f.close - 1 }));
        }
        // commentaires dans le corps (optionnel)
        if cfg.comments_in_function {
            for idx in f.open + 1..f.close.min(raw_lines.len()) {
                if raw_lines[idx].contains("//") || raw_lines[idx].contains("/*") {
                    add(&mut findings, idx + 1, 1, Severity::Info, "N-COMMENTFUNC",
                        "commentaire dans le corps d'une fonction", None);
                }
            }
        }
    }
    if !is_header && funcs.len() > cfg.max_functions_per_file {
        add(&mut findings, funcs[cfg.max_functions_per_file].start + 1, 1, Severity::Major,
            "N-FUNCCOUNT",
            format!("{} fonctions dans le fichier (max {})", funcs.len(), cfg.max_functions_per_file),
            None);
    }

    // ---- variables globales (heuristique)
    check_globals(&clean, &funcs, &mut findings);

    // ---- include guard pour les .h
    if is_header {
        check_include_guard(&clean, path, &mut findings);
    }

    // ---- C-A3 : le fichier doit se terminer par un retour à la ligne
    if !src.is_empty() && !src.ends_with('\n') {
        let last_line = raw_lines.len();
        add(&mut findings, last_line, 1, Severity::Info, "C-A3",
            "le fichier doit se terminer par un retour à la ligne",
            Some(Fix::InsertBefore {
                line: raw_lines.len(),
                content: String::new(),
            }));
    }

    findings.sort_by_key(|f| (f.line, f.col));
    findings.dedup_by(|a, b| a.line == b.line && a.col == b.col && a.rule == b.rule && a.message == b.message);
    findings
}

/// Trouve `kw(` collé (hors chaînes/commentaires, mot entier).
fn find_word_pattern(code: &str, kw: &str, _pat: &str) -> Option<usize> {
    let pat = format!("{kw}(");
    let mut from = 0;
    while let Some(rel) = code[from..].find(&pat) {
        let pos = from + rel;
        let before_ok = pos == 0
            || !code[..pos].chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_');
        if before_ok {
            return Some(pos);
        }
        from = pos + 1;
    }
    None
}

fn replace_word_pattern(raw: &str, kw: &str, with: &str) -> String {
    let pat = format!("{kw}(");
    let mut result = String::new();
    let mut from = 0;
    while let Some(rel) = raw[from..].find(&pat) {
        let pos = from + rel;
        let before_ok = pos == 0
            || !raw[..pos].chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_');
        if before_ok {
            result.push_str(&raw[from..pos]);
            result.push_str(with);
            from = pos + pat.len();
        } else {
            result.push_str(&raw[from..pos + 1]);
            from = pos + 1;
        }
    }
    result.push_str(&raw[from..]);
    result
}

fn find_keyword(code: &str, kw: &str) -> bool {
    find_word_pattern(code, kw, kw).is_some() || {
        let pat = format!("{kw} ");
        let mut from = 0;
        while let Some(rel) = code[from..].find(&pat) {
            let pos = from + rel;
            let before_ok = pos == 0
                || !code[..pos].chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_');
            if before_ok {
                return true;
            }
            from = pos + 1;
        }
        false
    }
}

/// Détecte un ternaire `cond ? a : b` (hors ??. pas en C).
fn has_ternary(code: &str) -> bool {
    let q = code.find('?');
    match q {
        Some(pos) => code[pos + 1..].contains(':'),
        None => false,
    }
}

/// Position d'un `;` suivi d'autre chose que des espaces/}` sur la ligne.
fn multi_statement_pos(code: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(rel) = code[from..].find(';') {
        let pos = from + rel;
        let rest = code[pos + 1..].trim();
        if !rest.is_empty() && !rest.starts_with('}') {
            return Some(pos + 1);
        }
        from = pos + 1;
    }
    None
}

/// Détecte `type* name` (étoile collée au type).
fn pointer_style_pos(code: &str) -> Option<usize> {
    for ty in ["int", "char", "void", "long", "short", "float", "double", "size_t", "ssize_t", "FILE"] {
        let pat = format!("{ty}* ");
        let mut from = 0;
        while let Some(rel) = code[from..].find(&pat) {
            let pos = from + rel;
            let before_ok = pos == 0
                || !code[..pos].chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_');
            if before_ok {
                return Some(pos + ty.len());
            }
            from = pos + 1;
        }
    }
    None
}

fn fix_pointer_style(raw: &str, line: usize) -> Fix {
    let mut content = raw.to_string();
    for ty in ["ssize_t", "size_t", "int", "char", "void", "long", "short", "float", "double", "FILE"] {
        let pat = format!("{ty}* ");
        if let Some(pos) = content.find(&pat) {
            let before_ok = pos == 0
                || !content[..pos].chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_');
            if before_ok {
                content = format!("{}{} *{}", &content[..pos], ty, &content[pos + pat.len()..]);
                break;
            }
        }
    }
    Fix::ReplaceLine { line, content }
}

fn check_header(raw_lines: &[&str], findings: &mut Vec<Finding>) {
    let head: String = raw_lines.iter().take(11).cloned().collect::<Vec<_>>().join("\n");
    let starts_comment = raw_lines.first().is_some_and(|l| l.trim() == "/*");
    // ancien format : « Made by » + « Login » ; nouveau : « EPITECH PROJECT » + « File description: »
    let old_format = head.contains("Made by") && head.contains("Login");
    let new_format = head.contains("EPITECH PROJECT") && head.contains("File description:");
    if !(starts_comment && (old_format || new_format)) {
        add(findings, 1, 1, Severity::Major, "N-HEADER",
            "en-tête Epitech manquant ou invalide en début de fichier", None);
    }
}

/// Heuristique : ligne de profondeur 0 ressemblant à une définition de variable
/// globale (hors prototypes, typedef, struct/enum, #define, extern).
fn check_globals(clean: &[String], funcs: &[FuncDef], findings: &mut Vec<Finding>) {
    let mut depth = 0i32;
    let mut func_bounds: Vec<(usize, usize)> =
        funcs.iter().map(|f| (f.start, f.close)).collect();
    func_bounds.sort();
    let mut in_struct = false;
    for (i, line) in clean.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("typedef") || t.starts_with("struct ") || t.starts_with("enum ") || t.starts_with("union ") {
            if t.ends_with('{') || t.ends_with("} ;") || t.contains('{') {
                in_struct = t.contains('{') && !t.contains('}');
            }
        }
        if depth == 0 && !in_struct && !t.is_empty() && !t.starts_with('#') {
            let in_func = func_bounds.iter().any(|(s, e)| i >= *s && i <= *e);
            let looks_global = !in_func
                && is_type_line(line)
                && !t.contains('(')
                && (t.ends_with(';') || t.contains('='))
                && !t.starts_with("extern ")
                && !t.starts_with("static const ")
                && !t.starts_with("const ");
            if looks_global {
                add(findings, i + 1, 1, Severity::Major, "N-GLOBAL",
                    "variable globale interdite en piscine", None);
            }
        }
        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
        if depth == 0 && in_struct && t.contains('}') {
            in_struct = false;
        }
    }
}

fn check_include_guard(clean: &[String], path: &Path, findings: &mut Vec<Finding>) {
    let text = clean.join("\n");
    let has_guard = text.contains("#ifndef") && text.contains("#define") && text.trim_end().ends_with("#endif");
    if !has_guard {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("HEADER")
            .to_uppercase()
            .replace(['-', '.'], "_");
        add(findings, 1, 1, Severity::Minor, "N-INCLUDEGUARD",
            format!("include guard manquant (attendu : {stem}_H)"), None);
    }
}

// ------------------------------------------------------------------ fichiers

/// Ajoute les problèmes web (html/css) d'un dossier au rapport,
/// comme des findings de règle « WEB » (info).
pub fn append_web_findings(dir: &Path, report: &mut Report) {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
            let mut findings = Vec::new();
            if ext == "html" || ext == "htm" {
                let src = std::fs::read_to_string(&p).unwrap_or_default();
                for f in crate::htmlcheck::check(&src) {
                    findings.push(Finding {
                        line: f.line, col: 1, severity: Severity::Info,
                        rule: "WEB", message: f.msg, fix: None,
                    });
                }
            } else if ext == "css" {
                let src = std::fs::read_to_string(&p).unwrap_or_default();
                for f in crate::csscheck::check(&src) {
                    findings.push(Finding {
                        line: f.line, col: 1, severity: Severity::Info,
                        rule: "WEB", message: f.msg, fix: None,
                    });
                }
            }
            if !findings.is_empty() {
                report.files.push(FileReport { path: p, findings });
            }
        }
    }
}

/// Vérifie un fichier ou un répertoire (récursif, .c/.h).
pub fn check_path(path: &Path, cfg: &NormeConfig) -> std::io::Result<Report> {
    let mut report = Report::default();
    let mut files = Vec::new();
    collect_sources(path, &mut files)?;
    files.sort();
    for file in files {
        let src = fs::read_to_string(&file)?;
        let findings = check_source(&file, &src, cfg);
        report.files.push(FileReport { path: file, findings });
    }
    Ok(report)
}

fn collect_sources(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            collect_sources(&p, out)?;
        } else if p.extension().is_some_and(|e| e == "c" || e == "h") {
            out.push(p);
        }
    }
    Ok(())
}

/// Applique les fixes en une passe par ligne DÉCROISSANTE (les index restent
/// valides pour les lignes déjà traitées). À ligne égale : remplace d'abord,
/// supprime ensuite, insère enfin — l'insertion pousse alors la ligne corrigée.
/// Retourne (nouveau contenu, nombre de corrections appliquées).
pub fn apply_fixes(src: &str, fixes: &[Fix]) -> (String, usize) {
    let mut lines: Vec<String> = src.lines().map(|l| l.to_string()).collect();
    let trailing_nl = src.ends_with('\n');

    fn line_of(f: &Fix) -> usize {
        match f {
            Fix::ReplaceLine { line, .. } => *line,
            Fix::InsertBefore { line, .. } => *line,
            Fix::DeleteLine { line } => *line,
        }
    }
    fn order(f: &Fix) -> u8 {
        match f {
            Fix::ReplaceLine { .. } => 0,
            Fix::DeleteLine { .. } => 1,
            Fix::InsertBefore { .. } => 2,
        }
    }
    let mut sorted: Vec<&Fix> = fixes.iter().collect();
    sorted.sort_by(|a, b| line_of(b).cmp(&line_of(a)).then(order(a).cmp(&order(b))));

    let mut applied = 0;
    for fix in sorted {
        match fix {
            Fix::ReplaceLine { line, content } if *line < lines.len() => {
                let repl: Vec<String> = content.split('\n').map(|s| s.to_string()).collect();
                lines.splice(*line..*line + 1, repl);
                applied += 1;
            }
            Fix::DeleteLine { line } if *line < lines.len() => {
                lines.remove(*line);
                applied += 1;
            }
            Fix::InsertBefore { line, content } if *line <= lines.len() => {
                lines.insert(*line, content.clone());
                applied += 1;
            }
            _ => {}
        }
    }
    let mut out = lines.join("\n");
    if trailing_nl {
        out.push('\n');
    }
    (out, applied)
}

/// Déduplique les fixes conflictuels (même ligne, plusieurs ReplaceLine).
pub fn dedup_fixes(findings: &[Finding]) -> Vec<Fix> {
    let mut by_line: std::collections::HashMap<usize, &Fix> = std::collections::HashMap::new();
    let mut others: Vec<Fix> = Vec::new();
    for f in findings {
        if let Some(fix) = &f.fix {
            match fix {
                Fix::ReplaceLine { line, .. } => {
                    by_line.entry(*line).or_insert(fix);
                }
                other => others.push(other.clone()),
            }
        }
    }
    by_line.into_values().cloned().chain(others).collect()
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> NormeConfig {
        NormeConfig::default()
    }

    fn check(src: &str) -> Vec<Finding> {
        check_source(Path::new("test.c"), src, &cfg())
    }

    const HEADER: &str = "/*\n** test.c for proj in /tmp\n**\n** Made by ev\n** Login <ev@epitech.eu>\n*/\n";

    #[test]
    fn colonnes_80() {
        let src = format!("{HEADER}int f(void)\n{{\n    int x; /* {} */\n    return (0);\n}}\n", "a".repeat(90));
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-COL80" && x.severity == Severity::Major));
    }

    #[test]
    fn for_interdit() {
        let src = format!("{HEADER}int f(void)\n{{\n    int i;\n\n    i = 0;\n    for (i = 0; i < 3; i++);\n    return (0);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-FOR"));
    }

    #[test]
    fn for_dans_chaine_pas_detecte() {
        let src = format!("{HEADER}int f(void)\n{{\n    char *s;\n\n    s = \"for (ever)\";\n    return (0);\n}}\n");
        let f = check(&src);
        assert!(!f.iter().any(|x| x.rule == "N-FOR"), "faux positif: {f:?}");
    }

    #[test]
    fn fonction_trop_longue() {
        let mut body = String::new();
        for i in 0..30 {
            body.push_str(&format!("    x += {i};\n"));
        }
        let src = format!("{HEADER}int f(int x)\n{{\n{body}    return (x);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-FUNC25" && x.severity == Severity::Major),
            "30 lignes de corps devraient etre signalees: {f:?}");
    }

    #[test]
    fn header_manquant() {
        let f = check("int main(void)\n{\n    return (0);\n}\n");
        assert!(f.iter().any(|x| x.rule == "N-HEADER"));
    }

    #[test]
    fn globale_detectee() {
        let src = format!("{HEADER}int g_count = 3;\n\nint f(void)\n{{\n    return (g_count);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-GLOBAL"), "globale non detectee: {f:?}");
    }

    #[test]
    fn trailing_space_fix() {
        let src = format!("{HEADER}int f(void)\n{{\n    int x;   \n\n    x = 1;\n    return (x);\n}}\n");
        let f = check(&src);
        let fixes = dedup_fixes(&f);
        let (new_src, applied) = apply_fixes(&src, &fixes);
        assert!(applied >= 1);
        assert!(!new_src.contains("int x;   "));
    }

    #[test]
    fn keyword_space_fix() {
        let src = format!("{HEADER}int f(int x)\n{{\n    if(x > 0)\n        return (1);\n    return (0);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-KEYWORDSPACE"));
        let fixes = dedup_fixes(&f);
        let (new_src, _) = apply_fixes(&src, &fixes);
        assert!(new_src.contains("if (x > 0)"), "fix non applique:\n{new_src}");
    }

    #[test]
    fn multi_statement() {
        let src = format!("{HEADER}int f(void)\n{{\n    int a;\n    int b;\n\n    a = 1; b = 2;\n    return (a + b);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-MULTISTMT"));
    }

    #[test]
    fn func_brace_and_count() {
        let mut src = HEADER.to_string();
        for i in 0..6 {
            src.push_str(&format!("int f{i}(void) {{\n    return (0);\n}}\n\n"));
        }
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-FUNCCOUNT"), "6 fonctions: {f:?}");
        assert!(f.iter().any(|x| x.rule == "N-FUNCBRACE"), "accolade inline: {f:?}");
    }

    #[test]
    fn pointer_style() {
        let src = format!("{HEADER}int f(void)\n{{\n    int* p;\n\n    p = 0;\n    return (p != 0);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-STARSTYLE"), "int* p non detecte: {f:?}");
    }

    #[test]
    fn newline_fin_fichier() {
        let src = format!("{HEADER}int f(void)\n{{\n    return (0);\n}}\n");
        assert!(!check(&src).iter().any(|x| x.rule == "C-A3"));
        let sans_nl = src.trim_end_matches('\n').to_string();
        let f = check(&sans_nl);
        assert!(f.iter().any(|x| x.rule == "C-A3"), "C-A3 non detecte: {f:?}");
        // et le fix rajoute le newline
        let fixes = dedup_fixes(&f);
        let (fixed, _) = apply_fixes(&sans_nl, &fixes);
        assert!(fixed.ends_with('\n'), "fix C-A3:\n{fixed:?}");
    }

    #[test]
    fn ternary_detected() {
        let src = format!("{HEADER}int f(int x)\n{{\n    return (x > 0 ? 1 : 0);\n}}\n");
        let f = check(&src);
        assert!(f.iter().any(|x| x.rule == "N-TERNARY"));
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;
    use std::path::Path;

    fn check(src: &str) -> Vec<Finding> {
        let cfg = NormeConfig::default();
        check_source(Path::new("t.c"), src, &cfg)
    }

    #[test]
    fn macro_continuation_pas_multistmt() {
        let src = "/*\n** EPITECH PROJECT, 2026\n** t\n** File description:\n** t\n*/\n\n#define M(x) do { \\\n    int y = (x); \\\n    f(y); \\\n} while (0)\n";
        let findings = check(src);
        assert!(
            !findings.iter().any(|f| f.rule == "N-MULTISTMT"),
            "les continuations de macro ne doivent pas être flaggées : {findings:?}"
        );
    }

    #[test]
    fn vrai_multistmt_flagge() {
        let src = "/*\n** EPITECH PROJECT, 2026\n** t\n** File description:\n** t\n*/\nint f(void)\n{\n    int a; a = 1;\n    return (0);\n}\n";
        let findings = check(src);
        assert!(
            findings.iter().any(|f| f.rule == "N-MULTISTMT"),
            "un vrai multi-statement doit être flaggé"
        );
    }
}

#[cfg(test)]
mod panic_regression_tests {
    use super::*;
    use std::path::Path;

    /// Une fonction à corps vide ou sur une ligne ne doit pas paniquer.
    #[test]
    fn corps_vide_ne_panique_pas() {
        let cfg = NormeConfig::default();
        for src in [
            "int f(void)\n{\n}\n",
            "int f(void) {}\n",
            "int f(void)\n{\n    return (0);\n}\n",
            "/*\n** EPITECH PROJECT, 2026\n** t\n** File description:\n** t\n*/\nint f(void)\n{\n}\n",
        ] {
            let _ = check_source(Path::new("t.c"), src, &cfg); // ne doit pas paniquer
        }
    }
}

#[cfg(test)]
mod robustness_tests {
    use super::*;
    use std::path::Path;

    /// Le checker ne doit JAMAIS paniquer, quel que soit l'entrée.
    #[test]
    fn aucune_panique_sur_cas_limites() {
        let cfg = NormeConfig::default();
        let cas: Vec<&str> = vec![
            "",
            "\n\n\n",
            "int f(void)",
            "int f(void)\n{\n",                    // jamais fermée
            "}\n}\n",                                 // fermantes seules
            "int f(void)\n{\n}\n}\n",                 // trop de fermantes
            "int f(int a, int b\n",                   // parenthèses ouvertes
            "{{{{{{",                                 // accolades seules
            "#define X(a) ((a) ? (a) : 0)\n",        // ternaire dans macro
            "char *s = \"texte avec ; et { et } \";\n", // chaîne avec ponctuation
            "int\tmain\t(\t)\t",                       // tabs partout
            "int f(void)\n{\n    return (0);",        // pas de fin
            "// juste un commentaire\n",
            "int f(void) { int a; a = 1; }",          // tout sur une ligne
        ];
        for (i, src) in cas.iter().enumerate() {
            // si ça panique, le test échoue avec le cas en cause
            let result = std::panic::catch_unwind(|| {
                let _ = check_source(Path::new("t.c"), src, &cfg);
            });
            assert!(result.is_ok(), "panique sur le cas #{i}: {src:?}");
        }
    }
}
