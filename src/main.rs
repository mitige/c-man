//! c-man — environnement d'apprentissage et de développement C pour la piscine.

mod tui;

use c_man::{compile, config, content, csscheck, htmlcheck, norme, progress, quiz, exo, render, serve};
use clap::{CommandFactory, Parser, Subcommand};
use clap::error::ErrorKind;
use content::{category_label, ENTRIES};
use render::Renderer;
use std::io::IsTerminal;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "c-man",
    version,
    about = "Ton environnement C de piscine : doc, norme, quiz, IA — dans le terminal",
    long_about = "c-man = documentation C + checker de Norme Epitech + compilation expliquée\n\
                  + quiz + suivi de progression + assistants IA (Kimi K3), le tout dans le TUI.\n\n\
                  Sans argument : environnement interactif (TUI).\n\
                  Avec un sujet : affiche la fiche directement, comme man."
)]
struct Cli {
    /// Sujet à afficher (ex: printf, malloc, pointeurs, la-norme)
    topic: Option<String>,

    /// Ouvre la TUI (sur le sujet si donné)
    #[arg(short, long)]
    tui: bool,

    /// Liste les fiches (optionnellement d'une catégorie)
    #[arg(short, long, value_name = "CATEGORIE", num_args = 0..=1)]
    list: Option<Option<String>>,

    /// Recherche plein texte dans les fiches
    #[arg(short, long, value_name = "REQUETE")]
    search: Option<String>,

    /// Affiche une fiche au hasard (le rituel du matin)
    #[arg(long)]
    random: bool,

    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Vérifie la Coding Style Epitech sur un fichier ou un dossier
    Check {
        /// Fichier .c/.h ou répertoire du projet
        path: String,
        /// Sortie JSON (pour scripts)
        #[arg(long)]
        json: bool,
    },
    /// Corrige automatiquement ce qui peut l'être (espaces, tabs, braces...)
    Fix {
        /// Fichier .c/.h ou répertoire du projet
        path: String,
        /// Affiche ce qui serait corrigé sans modifier les fichiers
        #[arg(long)]
        dry_run: bool,
    },
    /// Explique une règle de la Norme (ex: c-man explain N-COL80)
    Explain {
        /// Code de la règle (N-COL80, N-FUNC25, ...)
        rule: String,
    },
    /// Lance un quiz (catégorie optionnelle : langage, memoire, libc, outils, piscine)
    Quiz {
        /// Catégorie de questions
        category: Option<String>,
        /// Nombre de questions
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,
        /// Secondes par question (mode chrono)
        #[arg(long)]
        chrono: Option<u64>,
    },
    /// Affiche ta progression
    Progress,
    /// Crée un fichier .c norme-compliant (en-tête Epitech + squelette)
    New {
        /// Nom de l'exercice (ex: my_strlen)
        name: String,
        /// Prototype de la fonction (ex: "int my_strlen(char *s)")
        #[arg(short, long)]
        func: Option<String>,
    },
    /// Apprends un sujet sous tous ses angles (fiche + quiz + pièges + agent)
    Apprendre {
        /// Sujet (id de fiche, ex: pointeurs)
        sujet: String,
    },
    /// Build + lance les tests du projet (make test / criterion / main)
    Test {
        /// Répertoire du projet
        #[arg(default_value = ".")]
        path: String,
    },
    /// Vérifie ton environnement de piscine (outils, IA, dsh)
    Doctor,
    /// Sert le dossier courant en HTTP (prévisualiser ton web)
    Serve {
        /// Port de départ (défaut 8080, +1 si pris)
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Minuteur de focus (pomodoro) — découpe ta journée en blocs
    Focus {
        /// Minutes de focus (défaut 25)
        #[arg(default_value = "25")]
        minutes: u32,
    },
    /// Scaffolding web : index.html + style.css + script.js liés et prêts
    NewWeb {
        /// Nom du projet (dossier créé)
        nom: String,
    },
    /// Re-vérifie la norme à chaque modification (surveille le dossier)
    Watch {
        /// Dossier à surveiller
        #[arg(default_value = ".")]
        path: String,
    },
    /// Le rituel de livraison : add + commit + push en une commande
    Push {
        /// Message de commit
        #[arg(default_value = "travail du jour")]
        message: String,
    },
    /// Exercice guidé : la tâche, les indices, et la vérif réelle
    Exo {
        /// Id de l'exercice (ex: my_strlen) — vide pour la liste
        id: Option<String>,
        /// Compile + lance les tests de ta solution
        #[arg(long)]
        test: bool,
        /// Montre un indice (le niveau n, 1, 2…)
        #[arg(long)]
        indice: Option<usize>,
    },
    /// Scaffold un projet C complet (Makefile, include/, src/, en-têtes)
    NewProjet {
        /// Nom du projet
        nom: String,
    },
    /// Bilan du soir : ce que tu as fait + la suite pour demain
    Recap,
    /// Note personnelle sur une fiche (montre-la, ou écris-la)
    Note {
        /// Id de la fiche
        fiche: String,
        /// Le texte de la note (vide = afficher la note actuelle)
        texte: Option<String>,
    },
    /// Tu es bloqué ? Décris ton problème, on te débloque
    Bloque {
        /// Ton problème en quelques mots (ex: "mon malloc segfaulte")
        probleme: String,
    },
    /// Recherche unifiée : fiches + exercices + quiz + agents
    Apropos {
        /// Le mot recherché
        mot: String,
    },
    /// Journal d'apprentissage : écris ta réflexion du jour (ou relis)
    Journal {
        /// Ta réflexion (vide = lire le journal d'aujourd'hui)
        texte: Option<String>,
        /// Relis le journal d'un jour passé (offset en jours, 1 = hier)
        #[arg(short, long)]
        relire: Option<u32>,
    },
    /// Examen blanc : un test chronométré et noté (toutes catégories mélangées)
    Exam {
        /// Nombre de questions (défaut 10)
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,
        /// Minutes au total (défaut 15)
        #[arg(short = 't', long, default_value = "15")]
        minutes: u64,
    },
    /// La UNE chose à faire maintenant (réduit la charge de décision)
    Next,
    /// Session du matin : révision ciblée sur tes points fragiles
    Morning,
    /// Ouvre l'éditeur epitech-nano sur un fichier
    Edit {
        /// Fichier à ouvrir/créer
        path: Option<String>,
    },
}

fn renderer() -> Renderer {
    let is_tty = std::io::stdout().is_terminal();
    let width = if is_tty {
        crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80)
    } else {
        80
    };
    Renderer {
        color: is_tty && std::env::var_os("NO_COLOR").is_none(),
        width,
    }
}

/// Écrit sur stdout en survivant aux broken pipes (`c-man x | head`).
fn print_out(text: &str) {
    use std::io::Write;
    let mut stdout = std::io::stdout().lock();
    match stdout.write_all(text.as_bytes()).and_then(|_| stdout.flush()) {
        Ok(()) => {}
        Err(_) => std::process::exit(0),
    }
}

/// Titre court : la partie après « — » si présente.
fn short_title(entry: &content::Entry) -> &str {
    entry.title.split('—').nth(1).map(str::trim).unwrap_or(&entry.title)
}

fn show_topic(topic: &str) {
    match content::by_id(topic) {
        Some(entry) => {
            let mut out = renderer().render_entry(entry);
            // ta note personnelle, si elle existe
            if let Some(note) = progress::load().note(&entry.id) {
                out.push_str(&format!("\n  📝 Ta note : {note}\n"));
            }
            print_out(&out);
        }
        None => {
            let suggestions = content::search(topic);
            let mut msg = format!("c-man: aucune fiche pour « {topic} ».\n");
            if !suggestions.is_empty() {
                msg.push_str("\nVouliez-vous dire :\n");
                for (_, e) in suggestions.iter().take(5) {
                    msg.push_str(&format!("  {:<24} {}\n", e.id, short_title(e)));
                }
            }
            msg.push_str("\n`c-man -l` liste toutes les fiches.\n");
            eprint!("{msg}");
            std::process::exit(1);
        }
    }
}

fn run_tui(topic: Option<&str>) {
    if let Err(e) = tui::run(topic) {
        eprintln!("c-man: erreur TUI: {e}");
        std::process::exit(1);
    }
}

fn list_entries(filter: Option<&str>) {
    let mut out = String::new();
    let mut current_cat = String::new();
    let mut shown = 0;
    for entry in ENTRIES.iter() {
        if let Some(f) = filter {
            let f = f.to_lowercase();
            if entry.category != f
                && !entry.id.contains(&f)
                && !entry.title.to_lowercase().contains(&f)
            {
                continue;
            }
        }
        if entry.category != current_cat {
            current_cat = entry.category.clone();
            out.push_str(&format!("\n  {}\n", category_label(&current_cat)));
            out.push_str(&format!("  {}\n", "─".repeat(30)));
        }
        let day = entry
            .piscine_day
            .as_deref()
            .map(|d| format!(" [{d}]"))
            .unwrap_or_default();
        out.push_str(&format!(
            "    {:<24}{day}  {}\n",
            entry.id,
            short_title(entry)
        ));
        shown += 1;
    }
    if shown == 0 {
        if let Some(f) = filter {
            out.push_str(&format!("Aucune fiche pour le filtre « {f} ».\n"));
            out.push_str("Catégories : demarrer, langage, memoire, libc, outils, piscine.\n");
        }
    } else {
        out.push_str(&format!(
            "\n{shown} fiches. `c-man <sujet>` pour lire, `c-man` pour naviguer.\n"
        ));
    }
    print_out(&out);
}

fn search_entries(query: &str) {
    let results = content::search(query);
    if results.is_empty() {
        print_out(&format!("Aucun résultat pour « {query} ».\n"));
        return;
    }
    let mut out = format!("{} résultats pour « {query} » :\n\n", results.len());
    for (_, e) in results.iter().take(15) {
        out.push_str(&format!("  {:<24} {}\n", e.id, e.title));
    }
    out.push_str("\n`c-man <sujet>` pour lire une fiche.\n");
    print_out(&out);
}

fn random_entry() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    let entry = &ENTRIES[nanos % ENTRIES.len()];
    print_out(&format!("🎲 Fiche du hasard : {}\n", entry.id));
    show_topic(&entry.id.clone());
}

// ------------------------------------------------------------------ check / fix

fn cmd_check(path: &str, json: bool) -> ! {
    let cfg = config::load();
    let checker = cfg.norme.to_checker();
    let target = Path::new(path);
    if !target.exists() {
        eprintln!("c-man check: chemin introuvable : {path}");
        std::process::exit(2);
    }
    // fichier CSS → validateur CSS
    if target.is_file() && target.extension().is_some_and(|e| e == "css") {
        let src = std::fs::read_to_string(target).unwrap_or_default();
        let findings = csscheck::check(&src);
        if findings.is_empty() {
            print_out(&format!("\n  ✓ {path} : CSS valide.\n\n"));
            std::process::exit(0);
        }
        let mut out = format!("\n  {path}\n");
        for f in &findings {
            if f.line == 0 {
                out.push_str(&format!("    (global)  {}\n", f.msg));
            } else {
                out.push_str(&format!("    {:<5} {}\n", f.line, f.msg));
            }
        }
        out.push_str(&format!("\n  {} problème(s) CSS.\n\n", findings.len()));
        print_out(&out);
        std::process::exit(1);
    }
    // fichier JS → vérif syntaxe via node --check
    if target.is_file() && target.extension().is_some_and(|e| e == "js") {
        if !exo::is_node_available() {
            eprintln!("c-man check: node absent (sudo pacman -S nodejs) pour vérifier le JS.");
            std::process::exit(2);
        }
        let out = std::process::Command::new("node").arg("--check").arg(target).output();
        #[allow(unreachable_code)]
        return match out {
            Ok(o) if o.status.success() => {
                print_out(&format!("\n  ✓ {path} : JS syntaxiquement valide.\n\n"));
                std::process::exit(0);
            }
            Ok(o) => {
                eprintln!("\n  ✗ {path} : erreur de syntaxe JS\n{}\n", String::from_utf8_lossy(&o.stderr));
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("c-man check: {e}");
                std::process::exit(2);
            }
        };
    }
    // fichier HTML → validateur web (pas la norme C)
    if target.is_file() && target.extension().is_some_and(|e| e == "html" || e == "htm") {
        let src = std::fs::read_to_string(target).unwrap_or_default();
        let findings = htmlcheck::check(&src);
        if findings.is_empty() {
            print_out(&format!("\n  ✓ {path} : HTML valide.\n\n"));
            std::process::exit(0);
        }
        let mut out = format!("\n  {path}\n");
        for f in &findings {
            if f.line == 0 {
                out.push_str(&format!("    (global)  {}\n", f.msg));
            } else {
                out.push_str(&format!("    {:<5} {}\n", f.line, f.msg));
            }
        }
        out.push_str(&format!("\n  {} problème(s) HTML.\n\n", findings.len()));
        print_out(&out);
        std::process::exit(1);
    }
    // dossier → norme C + validation web (html/css/js) en un passage
    if target.is_dir() {
        let report = norme::check_path(target, &checker).unwrap_or_default();
        let majors = report.total(norme::Severity::Major) as u32;
        let minors = report.total(norme::Severity::Minor) as u32;
        let mut p = progress::load();
        p.record_norme_run(path, majors, minors);
        progress::save(&p);
        let mut out = String::new();
        let mut problems = majors + minors;
        // partie C
        if !report.files.is_empty() {
            if json {
                out.push_str(&report_json(&report));
            } else {
                out.push_str(&report_text(&report, path));
            }
        }
        // partie web (html/css/js)
        let web = check_web_dir(target);
        if !web.is_empty() {
            out.push_str("\n  ── web ──\n");
            for (file, msgs) in &web {
                problems += msgs.len() as u32;
                out.push_str(&format!("  {file}\n"));
                for m in msgs {
                    out.push_str(&format!("    {}\n", m));
                }
            }
        }
        if out.trim().is_empty() {
            out = format!("\n  ✓ {path} : tout est propre (C + web).\n");
        }
        print_out(&out);
        print_out("\n");
        std::process::exit(if problems > 0 { 1 } else { 0 });
    }
    match norme::check_path(target, &checker) {
        Ok(report) => {
            // enregistre dans la progression
            let mut p = progress::load();
            p.record_norme_run(
                path,
                report.total(norme::Severity::Major) as u32,
                report.total(norme::Severity::Minor) as u32,
            );
            progress::save(&p);

            if json {
                print_out(&report_json(&report));
            } else {
                print_out(&report_text(&report, path));
            }
            if report.total(norme::Severity::Major) > 0 {
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("c-man check: {e}");
            std::process::exit(2);
        }
    }
}

fn report_text(report: &norme::Report, path: &str) -> String {
    let r = renderer();
    let mut out = String::new();
    let (b, dim, red, yel, cyn, rst) = if r.color {
        ("\x1b[1m", "\x1b[2m", "\x1b[31m", "\x1b[33m", "\x1b[36m", "\x1b[0m")
    } else {
        ("", "", "", "", "", "")
    };
    for file in &report.files {
        if file.findings.is_empty() {
            continue;
        }
        out.push_str(&format!("\n{b}{}{rst}\n", file.path.display()));
        for f in &file.findings {
            let color = match f.severity {
                norme::Severity::Major => red,
                norme::Severity::Minor => yel,
                norme::Severity::Info => cyn,
            };
            let fixable = if f.fix.is_some() { " [fixable]" } else { "" };
            let code = match norme::official_code(f.rule) {
                Some(off) => format!("{} · {}", f.rule, off),
                None => f.rule.to_string(),
            };
            out.push_str(&format!(
                "  {:>4}:{:<3} {color}[{}]{rst} {}{dim} ({}){fixable}{rst}\n",
                f.line, f.col, f.severity, f.message, code
            ));
        }
    }
    let majors = report.total(norme::Severity::Major);
    let minors = report.total(norme::Severity::Minor);
    let infos = report.total(norme::Severity::Info);
    let fixable = report.fixable_count();
    out.push('\n');
    if report.findings_count() == 0 {
        out.push_str(&format!("{b}✓ {path} : aucune faute de style détectée.{rst}\n"));
    } else {
        out.push_str(&format!(
            "{b}{path}{rst} : {red}{majors} majeures{rst}, {yel}{minors} mineures{rst}, {infos} infos — {fixable} auto-corrigibles\n"
        ));
        out.push_str(&format!(
            "{dim}→ `c-man fix {path}` corrige les auto-corrigibles · `c-man explain <REGLE>` explique une règle{rst}\n"
        ));
    }
    out
}

fn report_json(report: &norme::Report) -> String {
    let files: Vec<serde_json::Value> = report
        .files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.path.display().to_string(),
                "findings": f.findings.iter().map(|x| serde_json::json!({
                    "line": x.line,
                    "col": x.col,
                    "severity": x.severity.to_string(),
                    "rule": x.rule,
                    "message": x.message,
                    "fixable": x.fix.is_some(),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({ "files": files })).unwrap()
}

fn cmd_fix(path: &str, dry_run: bool) -> ! {
    let cfg = config::load();
    let checker = cfg.norme.to_checker();
    let target = Path::new(path);
    if !target.exists() {
        eprintln!("c-man fix: chemin introuvable : {path}");
        std::process::exit(2);
    }
    let mut total_applied = 0;
    // itère jusqu'au point fixe : une ligne peut cumuler plusieurs fautes
    // (tab + espace après if...) dont chaque correction en révèle une autre.
    for round in 0..3 {
        let report = match norme::check_path(target, &checker) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("c-man fix: {e}");
                std::process::exit(2);
            }
        };
        let mut round_applied = 0;
        for file in &report.files {
            let fixes = norme::dedup_fixes(&file.findings);
            if fixes.is_empty() {
                continue;
            }
            let src = std::fs::read_to_string(&file.path).unwrap_or_default();
            let (new_src, applied) = norme::apply_fixes(&src, &fixes);
            if applied == 0 {
                continue;
            }
            if dry_run {
                if round == 0 {
                    print_out(&format!(
                        "— {} : {} correction(s) applicable(s)\n",
                        file.path.display(),
                        applied
                    ));
                }
            } else {
                if let Err(e) = std::fs::write(&file.path, &new_src) {
                    eprintln!("c-man fix: impossible d'écrire {}: {e}", file.path.display());
                    continue;
                }
                if round == 0 {
                    print_out(&format!(
                        "✓ {} : {} correction(s) appliquée(s)\n",
                        file.path.display(),
                        applied
                    ));
                }
            }
            round_applied += applied;
        }
        total_applied += round_applied;
        if dry_run || round_applied == 0 {
            break;
        }
    }
    // re-vérification après correction
    if !dry_run && total_applied > 0 {
        if let Ok(after) = norme::check_path(target, &checker) {
            print_out(&format!(
                "\nRe-vérification : {} faute(s) restante(s) ({} majeures). Les corrections non \
                 automatiques demandent ta main — `c-man check {path}` pour le détail, \
                 `c-man explain <REGLE>` pour comprendre.\n",
                after.findings_count(),
                after.total(norme::Severity::Major)
            ));
        }
    } else if dry_run {
        print_out(&format!(
            "\n(dry-run) {total_applied} correction(s) seraient appliquées. Sans --dry-run, c-man les applique puis re-vérifie.\n"
        ));
    } else {
        print_out("Rien à corriger automatiquement.\n");
    }
    std::process::exit(0);
}

fn cmd_explain(rule: &str) -> ! {
    let mut key = rule.to_uppercase();
    if !key.starts_with("N-") && !key.starts_with("C-") {
        key = format!("N-{key}");
    }
    // traduit un code officiel banana (C-F3, C-A3...) en règle interne
    if key.starts_with("C-") {
        if let Some(internal) = norme::official_to_internal(&key) {
            key = internal.to_string();
        }
    }
    let (titre, pourquoi, comment) = norme::explain_rule(&key);
    if titre == "Règle de style" {
        eprintln!("c-man explain: règle inconnue « {rule} ». Exemples : N-COL80, N-FUNC25, N-FOR, N-GLOBAL...");
        std::process::exit(1);
    }
    let r = renderer();
    let (b, rst) = if r.color { ("\x1b[1m", "\x1b[0m") } else { ("", "") };
    print_out(&format!(
        "\n{b}{key} — {titre}{rst}\n\nPourquoi : {pourquoi}\n\nComment corriger : {comment}\n"
    ));
    std::process::exit(0);
}

// ------------------------------------------------------------------ quiz / progress

fn cmd_quiz(category: Option<&str>, count: usize, chrono: Option<u64>) -> ! {
    let valid = ["langage", "memoire", "libc", "outils", "piscine", "bash", "web"];
    if let Some(c) = category {
        if !valid.contains(&c) {
            eprintln!("c-man quiz: catégorie inconnue « {c} » ({})", valid.join(", "));
            std::process::exit(2);
        }
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(1);
    let questions = quiz::pick(category, count, seed);
    if questions.is_empty() {
        eprintln!("c-man quiz: aucune question disponible.");
        std::process::exit(1);
    }
    let stdin = std::io::stdin();
    let mut correct = 0u32;
    let total = questions.len() as u32;
    print_out(&format!(
        "\nQuiz {} — {} questions. Réponds avec le numéro du choix.\n",
        category.unwrap_or("toutes catégories"),
        total
    ));
    for (i, q) in questions.iter().enumerate() {
        print_out(&format!(
            "\n── Question {}/{} ──\n{}\n",
            i + 1,
            total,
            q.question
        ));
        for (j, choice) in q.choices.iter().enumerate() {
            print_out(&format!("  {}. {}\n", j + 1, choice));
        }
        print_out("> ");
        let t0 = std::time::Instant::now();
        let mut answer = String::new();
        if stdin.read_line(&mut answer).is_err() {
            break;
        }
        let secs = t0.elapsed().as_secs();
        if let Some(limit) = chrono {
            if secs > limit {
                print_out(&format!("  ⏱ {}s (limite {}s) — trop lent, mais on continue.\n", secs, limit));
            }
        }
        let ok = answer
            .trim()
            .parse::<usize>()
            .ok()
            .is_some_and(|n| n == q.answer + 1);
        if ok {
            correct += 1;
            print_out("✓ Correct !\n");
        } else {
            print_out(&format!("✗ Raté — la bonne réponse était {}.\n", q.answer + 1));
            // pointe la fiche liée à relire
            if let Some(fiche) = q.related.first() {
                print_out(&format!("  → relis : c-man {fiche}\n"));
            }
        }
        print_out(&format!("  {}\n", q.explanation));
    }
    let mut p = progress::load();
    p.record_quiz(category.unwrap_or("general"), total, correct);
    progress::save(&p);
    let pct = if total > 0 { correct * 100 / total } else { 0 };
    print_out(&format!(
        "\n══ Score : {correct}/{total} ({pct} %) ══\n{}\n",
        match pct {
            90..=100 => "Solide. Niveau piscine validé sur ce thème.",
            70..=89 => "Bien ! Encore quelques fiches à relire.",
            50..=69 => "Mitigé : relis les fiches liées et retente.",
            _ => "Retourne aux fiches du thème, puis retente le quiz.",
        }
    ));
    std::process::exit(0);
}

fn cmd_progress() -> ! {
    let p = progress::load();
    let mut out = String::from("\n  📊 Ta progression c-man\n\n");
    // streak + activité de la semaine + focus
    let streak = p.streak();
    if streak > 0 {
        let flame = if streak >= 7 { "🔥" } else { "📅" };
        out.push_str(&format!("  {flame} Série : {streak} jour(s) d'affilée\n"));
    }
    out.push_str("  Cette semaine : ");
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        / 86_400
        * 86_400;
    for i in (0..7).rev() {
        let day = today - i * 86_400;
        out.push_str(if p.active_days.contains(&day) { "■" } else { "·" });
    }
    out.push('\n');
    if !p.focus_sessions.is_empty() {
        let total: u64 = p.focus_sessions.iter().sum();
        out.push_str(&format!(
            "  Focus : {} blocs, {} min de travail profond\n",
            p.focus_sessions.len(),
            total / 60
        ));
    }
    out.push('\n');
    if p.defended_count() > 0 {
        out.push_str(&format!(
            "  🎓 Notions défendues (tu sais les expliquer) : {}\n",
            p.defended_count()
        ));
    }
    out.push_str(&format!("  Fiches consultées : {} / {}\n", p.fiches.len(), ENTRIES.len()));
    for (cid, label) in content::CATEGORIES {
        let (seen, total) = p.category_coverage(cid, &ENTRIES);
        if total > 0 {
            let bar = progress_bar(seen, total, 20);
            out.push_str(&format!("    {label:<22} {bar} {seen}/{total}\n"));
        }
    }
    let total_q: u32 = p.quiz.iter().map(|q| q.total).sum();
    let correct: u32 = p.quiz.iter().map(|q| q.correct).sum();
    if total_q > 0 {
        out.push_str(&format!(
            "\n  Quiz : {} sessions, {correct}/{total_q} bonnes réponses ({} %)\n",
            p.quiz.len(),
            correct * 100 / total_q
        ));
    }
    let weak = p.weak_categories();
    if !weak.is_empty() {
        out.push_str("  Notions fragiles : ");
        out.push_str(
            &weak
                .iter()
                .map(|(c, ok, t)| format!("{c} ({ok}/{t})"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('\n');
    }
    let done: usize = p.exercises_done.values().map(|v| v.len()).sum();
    out.push_str(&format!("  Exercices marqués faits : {done}\n"));
    if let Some(last) = p.norme_runs.last() {
        out.push_str(&format!(
            "  Dernière norme : {} ({} majeures, {} mineures)\n",
            last.path, last.majors, last.minors
        ));
    }
    out.push('\n');
    print_out(&out);
    std::process::exit(0);
}

fn progress_bar(done: usize, total: usize, width: usize) -> String {
    let filled = if total > 0 { done * width / total } else { 0 };
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

// ------------------------------------------------------------------ ask / dsh



/// Session multi-angle sur un sujet : fiche, quiz lié, pièges, agent.
fn cmd_apprendre(sujet: &str) -> ! {
    // trouve la fiche (exact puis recherche)
    let entry = match content::by_id(sujet) {
        Some(e) => e,
        None => match content::search(sujet).first() {
            Some((_, e)) => e,
            None => {
                eprintln!("c-man apprendre: sujet inconnu « {sujet} ». `c-man -l` pour la liste.");
                std::process::exit(1);
            }
        },
    };
    let mut out = format!("\n  📖 {} — sous tous ses angles\n\n", entry.id);
    // 1. la fiche
    out.push_str(&format!("  1. LIS la fiche :        c-man {}\n", entry.id));
    // 2. quiz lié (questions dont related contient la fiche)
    let nb_quiz = quiz::QUESTIONS
        .iter()
        .filter(|q| q.related.contains(&entry.id))
        .count();
    if nb_quiz > 0 {
        out.push_str(&format!(
            "  2. TESTE-toi :           c-man quiz {}   ({} question(s) liée(s))\n",
            entry.category, nb_quiz
        ));
    } else {
        out.push_str(&format!("  2. TESTE-toi :           c-man quiz {}\n", entry.category));
    }
    // 3. les pièges
    if !entry.gotchas.is_empty() {
        out.push_str(&format!(
            "  3. ÉVITE les pièges :    {} piège(s) dans la fiche (lis la section PIÈGES)\n",
            entry.gotchas.len()
        ));
    }
    // 4. les exercices : d'abord l'exo guidé lié (vérifié), puis ceux de la fiche
    let exo_lie = exo::EXOS.iter().find(|e| e.fiche == entry.id);
    if let Some(ex) = exo_lie {
        out.push_str(&format!(
            "  4. PRATIQUE (vérifié) :  c-man exo {}   ← ton code est testé\n",
            ex.id
        ));
    }
    if !entry.exercises.is_empty() {
        out.push_str(&format!(
            "     + {} exercice(s) libre(s) dans la fiche (section EXERCICES)\n",
            entry.exercises.len()
        ));
    }
    // 5. approfondis avec les flashcards du sujet
    out.push_str("  5. APPROFONDIS : c-man, Ctrl+O, 6 (flashcards du sujet)\n");
    out.push_str("\n  L'ordre compte : lis → teste → pratique → explique. C'est comme ça\
                  \n  qu'on passe de « je crois savoir » à « je le ressens ».\n\n");
    print_out(&out);
    // enregistre la consultation
    let mut p = progress::load();
    p.record_view(&entry.id, 0);
    progress::save(&p);
    std::process::exit(0);
}


/// Build + teste le projet : make test, ou criterion (test_*.c), ou un main de démo.
fn cmd_test(path: &str) -> ! {
    let dir = Path::new(path);
    if !dir.is_dir() {
        eprintln!("c-man test: {path} n'est pas un dossier.");
        std::process::exit(2);
    }
    // 1. norme d'abord (une faute majeure et on le dit)
    let checker = config::load().norme.to_checker();
    if let Ok(rep) = norme::check_path(dir, &checker) {
        let majors = rep.total(norme::Severity::Major);
        if majors > 0 {
            print_out(&format!(
                "⚠ {} faute(s) MAJEURE(S) de norme — `c-man check {path}` pour le détail.\n\n",
                majors
            ));
        }
    }
    // 2. make test si la cible existe
    let has_make = dir.join("Makefile").exists() || dir.join("makefile").exists();
    if has_make {
        let out = std::process::Command::new("make")
            .arg("test")
            .current_dir(dir)
            .output();
        if let Ok(o) = out {
            let txt = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
            if o.status.success() {
                print_out(&format!("── make test ──\n{txt}\n✓ tests passés.\n"));
                std::process::exit(0);
            }
            if !txt.contains("No rule to make target") {
                print_out(&format!("── make test ──\n{txt}\n✗ échec.\n"));
                std::process::exit(1);
            }
            // pas de cible test → on continue avec les heuristiques
        }
    }
    // 3. criterion : des fichiers test_*.c / tests/ ?
    let mut test_srcs: Vec<_> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with("test_") && n.ends_with(".c") {
                test_srcs.push(e.path());
            }
        }
    }
    if !test_srcs.is_empty() {
        // compile avec criterion + les .c du projet (sauf main.c)
        let mut srcs: Vec<_> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().to_string();
                if n.ends_with(".c") && n != "main.c" && !n.starts_with("test_") {
                    srcs.push(e.path());
                }
            }
        }
        let mut cmd = std::process::Command::new("gcc");
        cmd.arg("-Wall").arg("-Wextra").arg("-std=c99").arg("-g")
            .arg("-o").arg("/tmp/c-man-test");
        for s in srcs.iter().chain(test_srcs.iter()) {
            cmd.arg(s);
        }
        cmd.arg("-lcriterion");
        let out = cmd.current_dir(dir).output();
        match out {
            Ok(o) if o.status.success() => {
                let run = std::process::Command::new("/tmp/c-man-test").output();
                match run {
                    Ok(r) => {
                        let txt = format!("{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr));
                        print_out(&format!("── criterion ──\n{txt}"));
                        std::process::exit(if r.status.success() { 0 } else { 1 });
                    }
                    Err(e) => { eprintln!("c-man test: {e}"); std::process::exit(2); }
                }
            }
            Ok(o) => {
                let txt = String::from_utf8_lossy(&o.stderr);
                print_out(&format!("── compile des tests : échec ──\n{txt}\n(criterion installé ? `c-man criterion`)\n"));
                std::process::exit(1);
            }
            Err(e) => { eprintln!("c-man test: {e}"); std::process::exit(2); }
        }
    }
    // 4. repli : compile + exécute main si présent
    let res = compile::compile_target(dir);
    print_out(&compile_text(&res));
    std::process::exit(if res.success { 0 } else { 1 });
}

/// Scanne un dossier pour les fichiers web (html/css/js) et les valide.
/// Retourne (fichier, messages) pour ceux qui ont des problèmes.
fn check_web_dir(dir: &Path) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
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
            } else {
                let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
                let mut msgs = Vec::new();
                if ext == "html" || ext == "htm" {
                    let src = std::fs::read_to_string(&p).unwrap_or_default();
                    for f in htmlcheck::check(&src) {
                        let loc = if f.line == 0 { "(global)".into() } else { format!("{}", f.line) };
                        msgs.push(format!("{loc}  {}", f.msg));
                    }
                } else if ext == "css" {
                    let src = std::fs::read_to_string(&p).unwrap_or_default();
                    for f in csscheck::check(&src) {
                        let loc = if f.line == 0 { "(global)".into() } else { format!("{}", f.line) };
                        msgs.push(format!("{loc}  {}", f.msg));
                    }
                } else if ext == "js" && exo::is_node_available() {
                    if let Ok(o) = std::process::Command::new("node").arg("--check").arg(&p).output() {
                        if !o.status.success() {
                            let err = String::from_utf8_lossy(&o.stderr);
                            // cherche la ligne "SyntaxError: ..." la plus parlante
                            let msg = err.lines()
                                .find(|l| l.contains("SyntaxError") || l.contains("Error"))
                                .unwrap_or("erreur de syntaxe");
                            let ligne = err.lines().next().unwrap_or("");
                            msgs.push(format!("js: {} — {}", ligne.trim(), msg.trim()));
                        }
                    }
                }
                if !msgs.is_empty() {
                    out.push((p.display().to_string(), msgs));
                }
            }
        }
    }
    out
}

/// Rend le résultat de compilation lisible.
fn compile_text(res: &compile::CompileResult) -> String {
    if res.diagnostics.is_empty() {
        return if res.success {
            "✓ compile propre.\n".into()
        } else {
            format!("✗ échec sans diagnostic.\n{}\n", res.raw)
        };
    }
    let mut out = String::new();
    for d in &res.diagnostics {
        let lvl = match d.level {
            compile::DiagLevel::Error => "ERREUR ",
            compile::DiagLevel::Warning => "warning",
            compile::DiagLevel::Note => "note   ",
        };
        out.push_str(&format!("  {}:{} {} {}\n", d.file, d.line, lvl, d.message));
        if let Some(h) = d.hint_text {
            out.push_str(&format!("      → {h}\n"));
        }
    }
    out
}

/// Vérifie l'environnement de piscine : outils, IA, dsh, config.
fn cmd_doctor() -> ! {
    let mut out = String::from("\n  🩺 c-man doctor — ton environnement de piscine\n\n");
    let check = |nom: &str, ok: bool, hint: &str| {
        format!("  {} {:<22} {}\n", if ok { "✓" } else { "✗" }, nom, if ok { "" } else { hint })
    };
    let has = |cmd: &str| std::process::Command::new("which").arg(cmd)
        .output().map(|o| o.status.success()).unwrap_or(false);
    // outils
    out.push_str(&check("gcc", has("gcc"), "→ sudo pacman -S gcc"));
    out.push_str(&check("make", has("make"), "→ sudo pacman -S make"));
    out.push_str(&check("gdb", has("gdb"), "→ sudo pacman -S gdb"));
    out.push_str(&check("valgrind", has("valgrind"), "→ sudo pacman -S valgrind"));
    out.push_str(&check("criterion (lib)", std::path::Path::new("/usr/include/criterion").exists(), "→ sudo pacman -S criterion"));
    out.push_str(&check("git", has("git"), "→ sudo pacman -S git"));
    // IA
    let cfg = config::load();
    // config identité pour les headers
    let ident_ok = cfg.name.is_some() || cfg.login.is_some();
    out.push_str(&check("identité (en-têtes)", ident_ok, "→ name/login dans ~/.config/c-man/config.toml"));
    out.push_str("\n");
    let manquants = out.matches("✗").count();
    if manquants == 0 {
        out.push_str("  Tout est prêt. Au travail.\n\n");
    } else {
        out.push_str(&format!("  {manquants} outil(s) à installer — les commandes sont à droite.\n\n"));
    }
    print_out(&out);
    std::process::exit(0);
}

/// Sert le dossier courant en HTTP jusqu'à Ctrl+C.
fn cmd_serve(port: u16) -> ! {
    let dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    match serve::serve(&dir, port) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("c-man serve: {e}");
            std::process::exit(2);
        }
    }
}


/// Minuteur de focus : un bloc de travail, puis une pause. Enregistré.
fn cmd_focus(minutes: u32) -> ! {
    if !(5..=120).contains(&minutes) {
        eprintln!("c-man focus: entre 5 et 120 minutes (25 est le bon réflexe).");
        std::process::exit(2);
    }
    print_out(&format!(
        "\n  🎯 Focus : {} minutes. Une seule tâche. Le reste attend.\n",
        minutes
    ));
    print_out("     (Ctrl+C pour arrêter — le bloc ne sera pas compté)\n\n");
    let start = std::time::Instant::now();
    let total = std::time::Duration::from_secs(minutes as u64 * 60);
    let mut last = 0u64;
    loop {
        let elapsed = start.elapsed();
        if elapsed >= total {
            break;
        }
        let remaining = total - elapsed;
        let m = remaining.as_secs() / 60;
        let s = remaining.as_secs() % 60;
        if remaining.as_secs() != last {
            eprint!("\r     ⏳ {:02}:{:02} restantes   ", m, s);
            use std::io::Write;
            let _ = std::io::stderr().flush();
            last = remaining.as_secs();
        }
        // vérifie Ctrl+C sans bloquer
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    let done_secs = start.elapsed().as_secs();
    print_out(&format!(
        "\n\n  ✓ Bloc terminé ({} min). Lève-toi, bois, regarde loin. 5 minutes.\n\n",
        done_secs / 60
    ));
    // enregistre la session de focus
    let mut p = progress::load();
    p.record_focus(done_secs);
    progress::save(&p);
    std::process::exit(0);
}


/// Scaffolding web : un projet HTML+CSS+JS lié, prêt pour `c-man serve`.
fn cmd_new_web(nom: &str) -> ! {
    let dir = Path::new(nom);
    if dir.exists() {
        eprintln!("c-man new-web: {nom} existe déjà.");
        std::process::exit(1);
    }
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("c-man new-web: {e}");
        std::process::exit(1);
    }
    let index = format!(
        "<!DOCTYPE html>\n<html lang=\"fr\">\n<head>\n    <meta charset=\"UTF-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    <title>{nom}</title>\n    <link rel=\"stylesheet\" href=\"style.css\">\n</head>\n<body>\n    <main>\n        <h1>{nom}</h1>\n    </main>\n    <script src=\"script.js\"></script>\n</body>\n</html>\n"
    );
    let css = "/* {0} — style */\n\nbody {\n    margin: 0;\n    font-family: sans-serif;\n}\n".replace("{0}", nom);
    let js = "// {0} — logique\n\ndocument.addEventListener('DOMContentLoaded', () => {\n    // prêt.\n});\n".replace("{0}", nom);
    let _ = std::fs::write(dir.join("index.html"), index);
    let _ = std::fs::write(dir.join("style.css"), css);
    let _ = std::fs::write(dir.join("script.js"), js);
    print_out(&format!(
        "✓ {nom}/ créé (index.html + style.css + script.js, déjà liés).\n  → cd {nom} && c-man serve   # puis ouvre http://127.0.0.1:8080\n\n"
    ));
    std::process::exit(0);
}

/// Surveille un dossier et re-vérifie la norme à chaque modification.
fn cmd_watch(path: &str) -> ! {
    let dir = Path::new(path);
    if !dir.is_dir() {
        eprintln!("c-man watch: {path} n'est pas un dossier.");
        std::process::exit(2);
    }
    let cfg = config::load().norme.to_checker();
    print_out(&format!("  👁 surveillance de {path} — la norme se re-vérifie à chaque sauvegarde (Ctrl+C pour arrêter)\n\n"));
    let mut derniers: std::collections::HashMap<String, std::time::SystemTime> = Default::default();
    let mut compte = 0u32;
    loop {
        // re-scan les .c/.h du dossier (non récursif + sous-dossiers directs)
        let mut fichiers: Vec<_> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() && p.extension().is_some_and(|x| x == "c" || x == "h") {
                    fichiers.push(p);
                }
            }
        }
        for f in fichiers {
            if let Ok(meta) = f.metadata() {
                if let Ok(mtime) = meta.modified() {
                    let key = f.display().to_string();
                    let avant = derniers.get(&key).copied();
                    if avant.is_none() || avant != Some(mtime) {
                        derniers.insert(key.clone(), mtime);
                        if avant.is_some() {
                            // modification détectée → re-check
                            if let Ok(src) = std::fs::read_to_string(&f) {
                                let findings = norme::check_source(&f, &src, &cfg);
                                let majors = findings.iter().filter(|x| x.severity == norme::Severity::Major).count();
                                compte += 1;
                                if majors == 0 {
                                    print_out(&format!("  ✓ {key} — propre (check #{compte})\n"));
                                } else {
                                    print_out(&format!("  ✗ {key} — {} majeure(s) (check #{compte}) : `c-man check {path}`\n", majors));
                                }
                            }
                        }
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
}

/// Le rituel de livraison piscine : add -A, commit, push, avec rappel norme.
fn cmd_push(message: &str) -> ! {
    let run = |args: &[&str]| -> Option<std::process::Output> {
        std::process::Command::new("git")
            .args(args)
            .output()
            .ok()
    };
    // pas un dépôt ?
    if run(&["rev-parse", "--git-dir"]).is_none_or(|o| !o.status.success()) {
        eprintln!("c-man push: pas un dépôt git ici.");
        std::process::exit(2);
    }
    // rappel norme avant de livrer
    let checker = config::load().norme.to_checker();
    if let Ok(rep) = norme::check_path(Path::new("."), &checker) {
        let majors = rep.total(norme::Severity::Major);
        if majors > 0 {
            print_out(&format!(
                "⚠ {} faute(s) MAJEURE(S) de norme — `c-man check .` avant de livrer !\n\n",
                majors
            ));
        }
    }
    print_out("── livraison\n");
    // add
    match run(&["add", "-A"]) {
        Some(o) if o.status.success() => {}
        _ => {
            eprintln!("c-man push: git add a échoué.");
            std::process::exit(1);
        }
    }
    // rien à commit ?
    if let Some(o) = run(&["status", "--porcelain"]) {
        if o.stdout.is_empty() {
            print_out("  rien à livrer (tout est déjà commité).\n");
            std::process::exit(0);
        }
    }
    // commit
    match run(&["commit", "-m", message]) {
        Some(o) if o.status.success() => {
            print_out(&format!("  ✓ commit « {} »\n", message));
        }
        Some(o) => {
            eprint!("{}", String::from_utf8_lossy(&o.stderr));
            std::process::exit(1);
        }
        None => {
            eprintln!("c-man push: git commit a échoué.");
            std::process::exit(1);
        }
    }
    // push
    match run(&["push"]) {
        Some(o) if o.status.success() => {
            print_out("  ✓ poussé. La moulinette le voit maintenant.\n\n");
            std::process::exit(0);
        }
        Some(o) => {
            eprintln!("  ✗ push échoué : {}", String::from_utf8_lossy(&o.stderr));
            std::process::exit(1);
        }
        None => {
            eprintln!("c-man push: git push a échoué.");
            std::process::exit(1);
        }
    }
}

/// Exercice guidé : pose la tâche, donne les indices, vérifie ta solution.
fn cmd_exo(id: Option<&str>, test: bool, indice: Option<usize>) -> ! {
    // liste des exercices
    let Some(id) = id else {
        let mut out = String::from("\n  Exercices guidés (le code est vérifié pour de vrai) :\n\n");
        for e in exo::EXOS {
            let fait = progress::load().exercises_done.contains_key(e.id);
            let langue = match e.lang { "js" => "·js", "sh" => "·sh", "html" => "·html", _ => "" };
            out.push_str(&format!(
                "  {} {:<26} {} {}\n",
                if fait { "✓" } else { " " },
                e.id,
                langue,
                e.title
            ));
        }
        out.push_str("\n  c-man exo <id> pour commencer.\n\n");
        print_out(&out);
        std::process::exit(0);
    };
    let Some(e) = exo::by_id(id) else {
        eprintln!("c-man exo: exercice inconnu « {id} ». `c-man exo` pour la liste.");
        std::process::exit(1);
    };
    // le dossier de travail de l'exo
    let dir = Path::new("exo").join(e.id);
    let fichier = dir.join(e.solution_name());

    // --test : compile + lance les tests
    if test {
        if !fichier.exists() {
            eprintln!("c-man exo: pas de solution — lance d'abord `c-man exo {id}`.");
            std::process::exit(2);
        }
        // exo bash : la solution devient sol.sh, le harnais l'exécute
        if e.lang == "sh" || e.lang == "html" {
            let sol = dir.join("sol.sh");
            std::fs::copy(&fichier, &sol).unwrap_or(0);
            let out = std::process::Command::new("bash")
                .arg("-c").arg(e.harness)
                .current_dir(&dir)
                .output();
            let (txt, ok) = match out {
                Ok(o) => (
                    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)),
                    o.status.success(),
                ),
                Err(err) => (format!("erreur: {err}"), false),
            };
            print_out(&format!("── tests de {id} (bash) ──\n{txt}"));
            if ok {
                print_out("\n  🎉 Tests passés. Exercice validé !\n\n");
                let mut p = progress::load();
                if !p.exercise_done(e.id, 0) { p.toggle_exercise(e.id, 0); }
                progress::save(&p);
                std::process::exit(0);
            }
            print_out("\n  Des tests échouent. Relis la tâche ou `c-man exo {id} --indice 1`.\n\n");
            std::process::exit(1);
        }
        // exo JS : on exécute solution + harnais avec node
        if e.lang == "js" {
            if !exo::is_node_available() {
                eprintln!("c-man exo: node n'est pas installé (sudo pacman -S nodejs).");
                std::process::exit(2);
            }
            let runner = dir.join("_run.js");
            let sol = std::fs::read_to_string(&fichier).unwrap_or_default();
            std::fs::write(&runner, format!("{sol}\n{}\n", e.harness)).unwrap();
            let out = std::process::Command::new("node").arg(&runner).output().unwrap();
            let txt = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            print_out(&format!("── tests de {id} (node) ──\n{txt}"));
            if out.status.success() {
                print_out("\n  🎉 Tests passés. Exercice validé !\n\n");
                let mut p = progress::load();
                if !p.exercise_done(e.id, 0) { p.toggle_exercise(e.id, 0); }
                progress::save(&p);
                std::process::exit(0);
            }
            print_out("\n  Des tests échouent. Relis la tâche ou `c-man exo {id} --indice 1`.\n\n");
            std::process::exit(1);
        }
        let testc = dir.join("_test.c");
        std::fs::write(&testc, e.harness).unwrap();
        let bin = dir.join("_t");
        let out = std::process::Command::new("gcc")
            .args(["-Wall", "-Wextra", "-Werror", "-std=c99", "-o"])
            .arg(&bin).arg(&fichier).arg(&testc)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let run = std::process::Command::new(&bin).output().unwrap();
                let txt = String::from_utf8_lossy(&run.stdout);
                print_out(&format!("── tests de {id} ──\n{txt}"));
                if run.status.success() {
                    print_out("\n  🎉 Tous les tests passent. Exercice validé !\n\n");
                    let mut p = progress::load();
                    if !p.exercise_done(e.id, 0) {
                        p.toggle_exercise(e.id, 0);
                    }
                    progress::save(&p);
                    std::process::exit(0);
                }
                print_out("\n  Des tests échouent. Relis la tâche, ou `c-man exo {id} --indice 1`.\n\n");
                std::process::exit(1);
            }
            Ok(o) => {
                eprintln!("── compile échouée ──\n{}", String::from_utf8_lossy(&o.stderr));
                std::process::exit(1);
            }
            Err(err) => {
                eprintln!("c-man exo: {err}");
                std::process::exit(2);
            }
        }
    }

    // prépare le fichier si besoin
    if !fichier.exists() {
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(&fichier, e.starter).unwrap();
    }
    // affiche la tâche
    let mut out = format!(
        "\n  ✍ Exercice : {}\n  {}\n\n  Prototype imposé :\n    {}\n\n",
        e.id, e.task, e.signature
    );
    out.push_str(&format!("  Ton fichier : {}\n", fichier.display()));
    out.push_str(&format!("  Vérifie :     c-man exo {} --test\n", e.id));
    out.push_str(&format!("  Théorie :     c-man {}\n", e.fiche));
    out.push_str(&format!("  Bloqué ?      c-man exo {} --indice 1\n\n", e.id));
    // indice demandé
    if let Some(n) = indice {
        if n >= 1 && n <= e.hints.len() {
            out.push_str(&format!("  💡 Indice {} : {}\n\n", n, e.hints[n - 1]));
        } else if n > e.hints.len() {
            out.push_str(&format!(
                "  Plus d'indices. Relis la fiche : c-man {}\n\n",
                e.id
            ));
        }
    }
    print_out(&out);
    std::process::exit(0);
}

/// Scaffold un projet C complet et norme-compliant.
fn cmd_new_projet(nom: &str) -> ! {
    let dir = Path::new(nom);
    if dir.exists() {
        eprintln!("c-man new-projet: {nom} existe déjà.");
        std::process::exit(1);
    }
    let cfg = config::load();
    let year = 2026;
    let auteur = cfg
        .name
        .as_deref()
        .zip(cfg.login.as_deref())
        .map(|(n, l)| format!("** Made by {n}\n** Login   <{l}@epitech.eu>\n"))
        .unwrap_or_default();
    let header = |desc: &str| {
        format!(
            "/*\n** EPITECH PROJECT, {year}\n** {nom}\n{auteur}** File description:\n** {desc}\n*/\n"
        )
    };
    // arborescence
    let _ = std::fs::create_dir_all(dir.join("src"));
    let _ = std::fs::create_dir_all(dir.join("include"));

    // Makefile (l'en-tête utilise des #, pas le bloc C /* */)
    let mk_header = format!(
        "# EPITECH PROJECT, {year}\n# {nom}\n# Makefile\n"
    );
    let makefile = format!(
        "{h}\nNAME\t=\t{nom}\n\nCC\t=\tgcc\nCFLAGS\t=\t-Wall -Wextra -Werror -std=c99 -I include\n\nSRC\t=\t$(wildcard src/*.c)\nOBJ\t=\t$(SRC:.c=.o)\n\nall:\t$(NAME)\n\n$(NAME):\t$(OBJ)\n\t$(CC) -o $(NAME) $(OBJ)\n\nclean:\n\trm -f $(OBJ)\n\nfclean:\tclean\n\trm -f $(NAME)\n\nre:\tfclean all\n\n.PHONY:\tall clean fclean re\n",
        h = mk_header
    );
    std::fs::write(dir.join("Makefile"), makefile).unwrap();

    // include/my.h avec guards
    let guard = nom.to_uppercase().replace(['-', '.'], "_");
    let h = format!(
        "{h}\n#ifndef {guard}_H\n    #define {guard}_H\n\n#endif\n",
        h = header("en-tête du projet"),
        guard = guard
    );
    std::fs::write(dir.join("include/my.h"), h).unwrap();

    // src/main.c
    let main = format!(
        "{h}\n#include \"my.h\"\n\nint main(int argc, char **argv)\n{{\n    (void)argc;\n    (void)argv;\n    return (0);\n}}\n",
        h = header("point d'entrée")
    );
    std::fs::write(dir.join("src/main.c"), main).unwrap();

    print_out(&format!(
        "✓ {nom}/ créé :\n    Makefile   (all/clean/fclean/re, -Werror, -I include)\n    include/my.h   (guards)\n    src/main.c\n  → cd {nom} && make\n\n"
    ));
    std::process::exit(0);
}


/// Bilan du soir : l'activité du jour + ce qui t'attend demain.
fn cmd_recap() -> ! {
    let p = progress::load();
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        / 86_400
        * 86_400;
    let tomorrow = today + 86_400;

    // activité d'aujourd'hui
    let fiches_jour: Vec<_> = p.fiches.iter()
        .filter(|(_, f)| f.last_seen >= today && f.last_seen < tomorrow)
        .collect();
    let quiz_jour = p.quiz.iter().filter(|q| q.at >= today && q.at < tomorrow).count();
    let focus_jour: u64 = p.focus_sessions.iter().filter(|_| p.active_days.contains(&today)).sum::<u64>();

    let mut out = String::from("\n  🌙 Bilan du jour\n\n");
    out.push_str(&format!("  Fiches lues : {}\n", fiches_jour.len()));
    if !fiches_jour.is_empty() {
        for (id, _) in fiches_jour.iter().take(5) {
            out.push_str(&format!("    · {id}\n"));
        }
    }
    if quiz_jour > 0 {
        out.push_str(&format!("  Quiz : {} session(s)\n", quiz_jour));
    }
    if focus_jour > 0 {
        out.push_str(&format!("  Focus : {} min\n", focus_jour / 60));
    }
    let streak = p.streak();
    out.push_str(&format!("  Série : {} jour(s) 🔥\n", streak));

    // demain
    out.push_str("\n  ── demain ──\n");
    let weak = p.weak_categories();
    if let Some((cat, _, _)) = weak.first() {
        out.push_str(&format!("  Point fragile à revoir : c-man quiz {cat}\n"));
    }
    if let Some(ex) = exo::EXOS.iter().find(|e| !p.exercise_done(e.id, 0)) {
        out.push_str(&format!("  Exercice : c-man exo {}\n", ex.id));
    }
    // une fiche liée à un point fragile
    let fiche = weak.first()
        .and_then(|(cat, _, _)| ENTRIES.iter().find(|e| e.category == *cat && !p.fiches.contains_key(&e.id)))
        .map(|e| e.id.as_str());
    if let Some(f) = fiche {
        out.push_str(&format!("  À lire : c-man {f}\n"));
    }
    out.push_str("\n  Bonne nuit. La constance paie.\n\n");
    print_out(&out);
    std::process::exit(0);
}

/// La prochaine action, une seule. Pas de choix à faire.
fn cmd_next() -> ! {
    let p = progress::load();
    // priorité : un exo non fait → un point fragile → une fiche non lue → flashcards
    let mut out = String::from("\n  👉 Une seule chose à faire :\n\n");

    if let Some(ex) = exo::EXOS.iter().find(|e| !p.exercise_done(e.id, 0)) {
        out.push_str(&format!("  c-man exo {}\n", ex.id));
        out.push_str(&format!("  ({})\n\n", ex.title));
        out.push_str("  C'est la seule chose. Les autres attendront.\n\n");
        print_out(&out);
        std::process::exit(0);
    }
    // un point fragile
    let weak = p.weak_categories();
    if let Some((cat, _, _)) = weak.first() {
        out.push_str(&format!("  c-man quiz {cat}\n"));
        out.push_str("  (ton point fragile — un quiz de 5 questions)\n\n");
        print_out(&out);
        std::process::exit(0);
    }
    // une fiche non lue
    let fiche = ENTRIES.iter().find(|e| !p.fiches.contains_key(&e.id));
    if let Some(f) = fiche {
        out.push_str(&format!("  c-man {}\n", f.id));
        out.push_str(&format!("  ({})\n\n", f.title));
        print_out(&out);
        std::process::exit(0);
    }
    out.push_str("  Tout est couvert. Flashcards : c-man, Ctrl+O, 8.\n\n");
    print_out(&out);
    std::process::exit(0);
}

/// Note personnelle sur une fiche.
fn cmd_note(fiche: &str, texte: Option<&str>) -> ! {
    let entry = match content::by_id(fiche).or_else(|| content::search(fiche).first().map(|(_, e)| *e)) {
        Some(e) => e,
        None => {
            eprintln!("c-man note: fiche inconnue « {fiche} ».");
            std::process::exit(1);
        }
    };
    let mut p = progress::load();
    match texte {
        // écrire / effacer
        Some(t) => {
            p.set_note(&entry.id, t);
            progress::save(&p);
            if t.trim().is_empty() {
                print_out(&format!("\n  Note effacée pour « {} ».\n\n", entry.id));
            } else {
                print_out(&format!("\n  📝 Note enregistrée pour « {} ».\n   Elle s'affichera à la lecture de la fiche.\n\n", entry.id));
            }
        }
        // lire
        None => {
            match p.note(&entry.id) {
                Some(n) => print_out(&format!("\n  📝 Ta note sur « {} » :\n\n  {}\n\n", entry.id, n)),
                None => print_out(&format!(
                "\n  Pas de note sur « {} ».\n  c-man note {} \"ta compréhension\"\n\n",
                entry.id, entry.id
            )),
            }
        }
    }
    std::process::exit(0);
}

/// Tu es bloqué : décris le problème, on te route vers la bonne aide.
fn cmd_bloque(probleme: &str) -> ! {
    let mut out = format!("\n  🆘 On te débloque : « {probleme} »\n\n");
    // 1. la fiche la plus pertinente — on cherche par mot-clé significatif
    let mots: Vec<&str> = probleme
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .collect();
    let mut best: Option<&c_man::content::Entry> = None;
    let mut best_score = i64::MIN;
    for mot in &mots {
        for (score, e) in content::search(mot) {
            if score > best_score {
                best_score = score;
                best = Some(e);
            }
        }
    }
    if let Some(e) = best {
        out.push_str(&format!("  1. Lis la fiche : c-man {}\n", e.id));
        // 2. un exo lié si ça touche un concept
        if let Some(ex) = exo::EXOS.iter().find(|x| x.fiche == e.id) {
            out.push_str(&format!("  2. Pratique : c-man exo {}\n", ex.id));
        }
    }
    // 3. l'agent qui aide (selon le type de problème)
    let bas = probleme.to_lowercase();
    let _agent = if bas.contains("segfault") || bas.contains("crash") || bas.contains("bug") || bas.contains("marche pas") {
        "debug"
    } else if bas.contains("malloc") || bas.contains("pointeur") || bas.contains("mémoire") || bas.contains("heap") {
        "memoire"
    } else if bas.contains("norme") || bas.contains("style") || bas.contains("propre") {
        "clean"
    } else if bas.contains("comment") || bas.contains("algo") || bas.contains("logique") {
        "algo"
    } else {
        "professeur"
    };

    out.push_str("\n  Tu n'es pas seul. Ces deux-là te débloquent.\n\n");
    print_out(&out);
    std::process::exit(0);
}

/// Recherche unifiée dans tout c-man : fiches, exercices, quiz, agents.
fn cmd_apropos(mot: &str) -> ! {
    let mot_l = mot.to_lowercase();
    let mut out = format!("\n  🔍 « {mot} » — partout dans c-man\n\n");

    // fiches
    let fiches = content::search(mot);
    if !fiches.is_empty() {
        out.push_str("  Fiches :\n");
        for (_, e) in fiches.iter().take(5) {
            out.push_str(&format!("    c-man {:<20} {}\n", e.id, e.title));
        }
        out.push('\n');
    }
    // exercices
    let exos: Vec<_> = exo::EXOS.iter()
        .filter(|e| e.id.to_lowercase().contains(&mot_l) || e.title.to_lowercase().contains(&mot_l) || e.task.to_lowercase().contains(&mot_l))
        .collect();
    if !exos.is_empty() {
        out.push_str("  Exercices :\n");
        for e in exos.iter().take(4) {
            out.push_str(&format!("    c-man exo {:<16} {}\n", e.id, e.title));
        }
        out.push('\n');
    }
    // questions de quiz
    let nb_quiz = quiz::QUESTIONS.iter()
        .filter(|q| q.question.to_lowercase().contains(&mot_l))
        .count();
    if nb_quiz > 0 {
        out.push_str(&format!("  Quiz : {} question(s) → c-man quiz\n\n", nb_quiz));
    }
    if fiches.is_empty() && exos.is_empty() && nb_quiz == 0 {
        out.push_str("  Rien trouvé. Essaie un autre mot, ou c-man bloque « … ».\n");
    }
    print_out(&out);
    print_out("\n");
    std::process::exit(0);
}

/// Journal d'apprentissage quotidien (~/.local/share/c-man/journal/).
fn cmd_journal(texte: Option<&str>, relire: Option<u32>) -> ! {
    let dir = dirs_home().join("journal");
    let _ = std::fs::create_dir_all(&dir);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let jour = now / 86_400;

    // relire un jour passé
    if let Some(offset) = relire {
        let cible = jour.saturating_sub(offset as u64);
        let f = dir.join(format!("{cible}.md"));
        if f.exists() {
            let content = std::fs::read_to_string(&f).unwrap_or_default();
            print_out(&format!("\n  📓 Journal du jour il y a {offset} jour(s) :\n\n{}\n\n", content));
        } else {
            print_out(&format!("\n  Pas de journal pour il y a {offset} jour(s).\n  Écris aujourd'hui : c-man journal \"ce que j'ai compris…\"\n\n"));
        }
        std::process::exit(0);
    }

    let f = dir.join(format!("{jour}.md"));
    match texte {
        // écrire / ajouter
        Some(t) => {
            let mut content = std::fs::read_to_string(&f).unwrap_or_default();
            if !content.is_empty() {
                content.push_str("\n\n");
            }
            content.push_str(t);
            std::fs::write(&f, content).unwrap_or_default();
            print_out("\n  📓 Noté dans ton journal du jour.\n  `c-man journal` pour le relire, `c-man journal -r 1` pour hier.\n\n");
        }
        // lire aujourd'hui
        None => {
            if f.exists() {
                let content = std::fs::read_to_string(&f).unwrap_or_default();
                print_out(&format!("\n  📓 Ton journal d'aujourd'hui :\n\n{}\n\n", content));
            } else {
                print_out("\n  📓 Pas encore de journal aujourd'hui.\n  `c-man journal \"ce que j'ai compris, ce qui m'a bloqué\"`\n  Écrire avec tes mots, c'est ancrer.\n\n");
            }
        }
    }
    std::process::exit(0);
}

/// Le dossier de données c-man.
fn dirs_home() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".local/share/c-man")
}

/// Examen blanc : questions mélangées, chrono global, note finale.
fn cmd_exam(count: usize, minutes: u64) -> ! {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(1);
    let questions = quiz::pick(None, count, seed);
    if questions.is_empty() {
        eprintln!("c-man exam: aucune question.");
        std::process::exit(1);
    }
    let budget = std::time::Duration::from_secs(minutes * 60);
    let debut = std::time::Instant::now();
    let stdin = std::io::stdin();
    let mut correct = 0u32;
    let total = questions.len() as u32;

    print_out(&format!(
        "\n  📝 EXAMEN BLANC — {total} questions, {minutes} minutes.\n  Réponds avec le numéro. Bonne chance.\n"
    ));
    for (i, q) in questions.iter().enumerate() {
        let reste = budget.saturating_sub(debut.elapsed());
        if reste.is_zero() {
            print_out("\n  ⏱ Temps écoulé !\n");
            break;
        }
        let m = reste.as_secs() / 60;
        let s2 = reste.as_secs() % 60;
        print_out(&format!(
            "\n── Q{}/{}  ({:02}:{:02} restantes) ──\n{}\n",
            i + 1, total, m, s2, q.question
        ));
        for (j, choice) in q.choices.iter().enumerate() {
            print_out(&format!("  {}. {}\n", j + 1, choice));
        }
        print_out("> ");
        let mut answer = String::new();
        if stdin.read_line(&mut answer).is_err() {
            break;
        }
        if answer.trim().parse::<usize>().ok() == Some(q.answer + 1) {
            correct += 1;
            print_out("  ✓\n");
        } else {
            print_out(&format!("  ✗ (c'était {})\n", q.answer + 1));
        }
    }
    let pct = if total > 0 { correct * 100 / total } else { 0 };
    // partie code : un exercice à faire sous pression (le vrai exam en a)
    let p0 = progress::load();
    if let Some(ex) = exo::EXOS.iter().filter(|e| e.lang == "c").find(|e| !p0.exercise_done(e.id, 0)) {
        print_out(&format!(
            "\n  ── partie code ──\n  Le vrai exam a du code. Tu as un exercice :\n\n  c-man exo {}\n  ({})\n\n  Fais-le maintenant, puis `c-man exo {} --test`.\n",
            ex.id, ex.title, ex.id
        ));
    }
    let note = match pct {
        90..=100 => "Excellent — niveau tête de promo.",
        70..=89 => "Solide. Encore un peu.",
        50..=69 => "Ça passe, mais bosse les points fragiles.",
        _ => "À retravailler — c-man morning demain.",
    };
    print_out(&format!(
        "\n  ══ NOTE : {correct}/{total} ({pct} %) ══\n  {note}\n\n"
    ));
    let mut p = progress::load();
    p.record_quiz("exam", total, correct);
    progress::save(&p);
    std::process::exit(0);
}

/// Session du matin : accueil, points fragiles, fiche à relire, quiz ciblé.
fn cmd_morning() -> ! {
    let p = progress::load();
    let mut out = String::from("\n  Bonjour. Session du matin — court, ciblé, utile.\n\n");
    let weak = p.weak_categories();
    if !weak.is_empty() {
        out.push_str("  Tes points fragiles (d'après tes quiz) :\n");
        for (cat, ok, total) in &weak {
            out.push_str(&format!("    · {cat} : {ok}/{total}\n"));
        }
        out.push('\n');
    }
    // fiche à relire : la moins vue de la catégorie la plus fragile
    let target_cat = weak.first().map(|(c, _, _)| c.as_str());
    let fiche = ENTRIES
        .iter()
        .filter(|e| target_cat.is_none_or(|c| e.category == c))
        .min_by_key(|e| p.fiches.get(&e.id).map(|s| s.views).unwrap_or(0))
        .map(|e| e.id.as_str());
    if let Some(f) = fiche {
        out.push_str(&format!("  À relire aujourd'hui : c-man {f}\n"));
    }
    if let Some((cat, _, _)) = weak.first() {
        out.push_str(&format!("  Quiz ciblé : c-man quiz {cat}\n"));
    }
    // flashcards à revoir (répétition espacée)
    let a_revoir = quiz::flashcards()
        .iter()
        .filter(|(fiche, _)| p.flash_priority(fiche).0 == 0)
        .count();
    if a_revoir > 0 {
        out.push_str(&format!(
            "  Flashcards à revoir : {} (les pièges qui t'ont fait bloquer)\n",
            a_revoir
        ));
    }
    out.push_str("  Révision éclair : c-man, Ctrl+O, 8 (flashcards)\n");
    // suggère un exercice guidé non fait
    if let Some(ex) = exo::EXOS.iter().find(|e| !p.exercise_done(e.id, 0)) {
        out.push_str(&format!("  Exercice du jour : c-man exo {}\n", ex.id));
    }
    // une notion lue mais pas encore revue → relis-la
    if let Some(e) = ENTRIES.iter().find(|e| p.fiches.contains_key(&e.id)) {
        let _ = e;
    }
    let streak = p.streak();
    if streak > 0 {
        let flame = if streak >= 7 { "🔥" } else { "📅" };
        out.push_str(&format!(
            "  {} Série : {} jour(s) d'affilée{}.\n",
            flame,
            streak,
            if streak >= 7 { " — impressionnant" } else { "" }
        ));
    }
    if !p.focus_sessions.is_empty() {
        let total: u64 = p.focus_sessions.iter().sum();
        out.push_str(&format!(
            "  Focus : {} blocs jusqu'ici ({} min de travail profond).\n",
            p.focus_sessions.len(),
            total / 60
        ));
    }
    if p.quiz.is_empty() && p.fiches.is_empty() {
        out.push_str("\n  (Pas encore de données — commence par c-man roadmap, et reviens demain.)\n");
    }
    out.push('\n');
    print_out(&out);
    std::process::exit(0);
}


/// Scaffolding : crée <nom>.c avec en-tête Epitech + squelette Norme.
fn cmd_new(name: &str, func: Option<&str>) -> ! {
    let cfg = config::load();
    let path = format!("{name}.c");
    if std::path::Path::new(&path).exists() {
        eprintln!("c-man new: {path} existe déjà.");
        std::process::exit(1);
    }
    let year = 2026; // année de la promo courante
    // identité depuis la config (name/login) si définie
    let auteur = cfg
        .name
        .as_deref()
        .zip(cfg.login.as_deref())
        .map(|(n, l)| format!("** Made by {n}\n** Login   <{l}@epitech.eu>\n"))
        .unwrap_or_default();
    let mut content = format!(
        "/*\n** EPITECH PROJECT, {year}\n** {name}\n{auteur}** File description:\n** {name}\n*/\n\n"
    );
    if cfg.name.is_none() {
        eprintln!("(astuce : définis name+login dans ~/.config/c-man/config.toml pour l'en-tête complet)");
    }
    if let Some(f) = func {
        let f = f.trim().trim_end_matches(';');
        // corps minimal conforme : accolade sur sa ligne ; retour par défaut
        // adapté au type (return; dans une fonction non-void casse -Werror)
        let retourne_valeur = !f.trim_start().starts_with("void");
        let retour = if retourne_valeur { "    return (0);" } else { "    return;" };
        // marque chaque paramètre comme (void) pour passer -Werror=unused-parameter
        let mut corps = String::new();
        if let Some(p) = f.find('(') {
            if let Some(q) = f.rfind(')') {
                let params = &f[p + 1..q];
                for param in params.split(',') {
                    let param = param.trim();
                    if param == "void" || param.is_empty() {
                        continue;
                    }
                    if let Some(nom) = param.split_whitespace().last() {
                        let nom = nom.trim_start_matches('*');
                        corps.push_str(&format!("    (void){nom};\n"));
                    }
                }
            }
        }
        content.push_str(&format!("{f}\n{{\n{corps}{retour}\n}}\n"));
    }
    if let Err(e) = std::fs::write(&path, &content) {
        eprintln!("c-man new: impossible d'écrire {path}: {e}");
        std::process::exit(1);
    }
    print_out(&format!(
        "✓ {path} créé (en-tête Epitech, Norme-clean).\n  → `c-man edit {path}` pour coder.\n"
    ));
    std::process::exit(0);
}



// ------------------------------------------------------------------ main

fn main() {
    let _ = config::ensure_exists();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) if e.kind() == ErrorKind::DisplayHelp || e.kind() == ErrorKind::DisplayVersion => {
            e.print().expect("affichage aide");
            return;
        }
        Err(e) => {
            e.print().expect("affichage erreur");
            std::process::exit(2);
        }
    };

    if ENTRIES.is_empty() {
        eprintln!("c-man: aucune fiche embarquée. Réinstalle le paquet.");
        std::process::exit(1);
    }

    match cli.command {
        Some(Cmd::Check { path, json }) => cmd_check(&path, json),
        Some(Cmd::Fix { path, dry_run }) => cmd_fix(&path, dry_run),
        Some(Cmd::Explain { rule }) => cmd_explain(&rule),
        Some(Cmd::Quiz { category, count, chrono }) => cmd_quiz(category.as_deref(), count, chrono),
        Some(Cmd::Progress) => cmd_progress(),
        Some(Cmd::New { name, func }) => cmd_new(&name, func.as_deref()),
        Some(Cmd::Apprendre { sujet }) => cmd_apprendre(&sujet),
        Some(Cmd::Test { path }) => cmd_test(&path),
        Some(Cmd::Doctor) => cmd_doctor(),
        Some(Cmd::Serve { port }) => cmd_serve(port),
        Some(Cmd::Focus { minutes }) => cmd_focus(minutes),
        Some(Cmd::NewWeb { nom }) => cmd_new_web(&nom),
        Some(Cmd::Watch { path }) => cmd_watch(&path),
        Some(Cmd::Push { message }) => cmd_push(&message),
        Some(Cmd::Exo { id, test, indice }) => cmd_exo(id.as_deref(), test, indice),
        Some(Cmd::NewProjet { nom }) => cmd_new_projet(&nom),
        Some(Cmd::Recap) => cmd_recap(),
        Some(Cmd::Note { fiche, texte }) => cmd_note(&fiche, texte.as_deref()),
        Some(Cmd::Bloque { probleme }) => cmd_bloque(&probleme),
        Some(Cmd::Apropos { mot }) => cmd_apropos(&mot),
        Some(Cmd::Journal { texte, relire }) => cmd_journal(texte.as_deref(), relire),
        Some(Cmd::Exam { count, minutes }) => cmd_exam(count, minutes),
        Some(Cmd::Next) => cmd_next(),
        Some(Cmd::Morning) => cmd_morning(),
        Some(Cmd::Edit { path }) => {
            let p = path.map(std::path::PathBuf::from);
            if let Err(e) = c_man::editor::run(p) {
                eprintln!("c-man edit: {e}");
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        None => {}
    }

    if cli.random {
        random_entry();
    } else if let Some(filter) = cli.list {
        list_entries(filter.as_deref());
    } else if let Some(query) = cli.search {
        search_entries(&query);
    } else if cli.tui {
        run_tui(cli.topic.as_deref());
    } else if let Some(topic) = cli.topic {
        show_topic(&topic);
    } else if std::io::stdout().is_terminal() && std::io::stdin().is_terminal() {
        run_tui(None);
    } else {
        Cli::command().print_help().expect("aide");
        println!();
    }
}
