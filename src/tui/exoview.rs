//! Écran des exercices guidés (TUI).

use super::*;
use crate::exo;
use ratatui::widgets::{Paragraph, Wrap};

pub(crate) struct ExoState {
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
    /// résultat du dernier test (titre, lignes, succès)
    pub(crate) result: Option<(String, Vec<String>, bool)>,
    pub(crate) busy: bool,
}

impl ExoState {
    pub(crate) fn new() -> Self {
        Self {
            selected: 0,
            scroll: 0,
            result: None,
            busy: false,
        }
    }
}

pub(crate) fn on_key(shell: &mut Shell, key: KeyEvent) {
    let st = &mut shell.exo;
    if st.busy {
        if key.code == KeyCode::Esc {
            st.busy = false;
        }
        return;
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if st.selected + 1 < exo::EXOS.len() {
                st.selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            st.selected = st.selected.saturating_sub(1);
        }
        // lance le test de l'exercice sélectionné (async)
        KeyCode::Enter => {
            let e = &exo::EXOS[st.selected];
            let id = e.id.to_string();
            st.busy = true;
            let tx = shell.tx.clone();
            std::thread::spawn(move || {
                let result = run_exo_test(&id);
                let _ = tx.send(AsyncMsg::ExoResult(result));
            });
        }
        _ => {}
    }
}

/// Exécute le test d'un exercice (solution dans ./exo/<id>/).
fn run_exo_test(id: &str) -> (String, Vec<String>, bool) {
    let Some(e) = exo::by_id(id) else {
        return (id.into(), vec!["exercice inconnu".into()], false);
    };
    let dir = std::path::Path::new("exo").join(e.id);
    let fichier = dir.join(e.solution_name());
    let mut out = Vec::new();
    if !fichier.exists() {
        // prépare le starter
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(&fichier, e.starter).unwrap_or_default();
        out.push(format!("squelette créé : {}", fichier.display()));
        out.push("écris ta solution puis Entrée pour re-tester.".into());
        return (e.id.into(), out, false);
    }
    if e.lang == "sh" || e.lang == "html" {
        let sol = dir.join("sol.sh");
        std::fs::copy(&fichier, &sol).unwrap_or(0);
        let o = std::process::Command::new("bash")
            .arg("-c").arg(e.harness)
            .current_dir(&dir)
            .output();
        return match o {
            Ok(r) => {
                let txt = format!("{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr));
                let ok = r.status.success();
                out.extend(txt.lines().map(|l| l.to_string()));
                out.push(if ok { "✓ exercice validé".into() } else { "des tests échouent".into() });
                if ok {
                    let mut p = crate::progress::load();
                    if !p.exercise_done(e.id, 0) { p.toggle_exercise(e.id, 0); }
                    crate::progress::save(&p);
                }
                (e.id.into(), out, ok)
            }
            Err(err) => (e.id.into(), vec![format!("erreur: {err}")], false),
        };
    }
    if e.lang == "js" {
        if !exo::is_node_available() {
            return (e.id.into(), vec!["node n'est pas installé".into()], false);
        }
        let runner = dir.join("_run.js");
        let sol = std::fs::read_to_string(&fichier).unwrap_or_default();
        let _ = std::fs::write(&runner, format!("{sol}\n{}\n", e.harness));
        let o = std::process::Command::new("node").arg(&runner).output();
        return match o {
            Ok(r) => {
                let txt = format!(
                    "{}{}",
                    String::from_utf8_lossy(&r.stdout),
                    String::from_utf8_lossy(&r.stderr)
                );
                let ok = r.status.success();
                out.extend(txt.lines().map(|l| l.to_string()));
                out.push(if ok { "✓ exercice validé".into() } else { "des tests échouent".into() });
                (e.id.into(), out, ok)
            }
            Err(err) => (e.id.into(), vec![format!("erreur: {err}")], false),
        };
    }
    // C : compile + run
    let testc = dir.join("_test.c");
    std::fs::write(&testc, e.harness).unwrap_or_default();
    let bin = dir.join("_t");
    let comp = std::process::Command::new("gcc")
        .args(["-Wall", "-Wextra", "-Werror", "-std=c99", "-o"])
        .arg(&bin)
        .arg(&fichier)
        .arg(&testc)
        .output();
    match comp {
        Ok(o) if o.status.success() => {
            let r = std::process::Command::new(&bin).output().unwrap();
            let txt = String::from_utf8_lossy(&r.stdout);
            let ok = r.status.success();
            out.extend(txt.lines().map(|l| l.to_string()));
            out.push(if ok { "✓ exercice validé".into() } else { "des tests échouent".into() });
            if ok {
                let mut p = crate::progress::load();
                if !p.exercise_done(e.id, 0) {
                    p.toggle_exercise(e.id, 0);
                }
                crate::progress::save(&p);
            }
            (e.id.into(), out, ok)
        }
        Ok(o) => {
            out.push("compile échouée :".into());
            out.extend(
                String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .take(8)
                    .map(|l| l.to_string()),
            );
            (e.id.into(), out, false)
        }
        Err(err) => (e.id.into(), vec![format!("erreur: {err}")], false),
    }
}

pub(crate) fn on_exo_result(shell: &mut Shell, result: (String, Vec<String>, bool)) {
    shell.exo.busy = false;
    shell.exo.result = Some(result);
    shell.exo.scroll = 0;
}

pub(crate) fn draw(frame: &mut Frame, shell: &mut Shell, area: Rect) {
    let palette = *shell.palette();
    let st = &shell.exo;
    let accent = Style::default().fg(palette.accent());
    let dim = Style::default().fg(palette.dim());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    // liste des exercices
    let prog = crate::progress::load();
    let mut items: Vec<Line> = Vec::new();
    for (i, e) in exo::EXOS.iter().enumerate() {
        let fait = prog.exercise_done(e.id, 0);
        let mark = if fait { "✓" } else { " " };
        let langue = match e.lang { "js" => "·js", "sh" => "·sh", "html" => "·html", _ => "" };
        let style = if i == st.selected {
            accent.add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if i == st.selected { "▸ " } else { "  " };
        // difficulté discrète (1-5 points)
        let diff = "●".repeat(e.difficulty as usize) + &"○".repeat(5 - e.difficulty as usize);
        items.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{mark} {}{}", e.id, langue), style),
            Span::styled(format!("  {diff}"), Style::default().fg(palette.dim())),
        ]));
    }
    frame.render_widget(
        Paragraph::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(dim)
                .title(Span::styled("  exercices  ", Style::default().fg(palette.accent()))),
        ),
        chunks[0],
    );

    // détail de l'exercice sélectionné
    let e = &exo::EXOS[st.selected];
    let mut right: Vec<Line> = vec![
        Line::from(Span::styled(e.title.to_string(), accent.add_modifier(Modifier::BOLD))),
        Line::default(),
        Line::from(e.task),
        Line::default(),
        Line::from(Span::styled(format!("prototype : {}", e.signature), dim)),
        Line::from(Span::styled(format!("fichier : exo/{}/{}", e.id, e.solution_name()), dim)),
        Line::default(),
    ];
    if st.busy {
        right.push(Line::from(Span::styled("⏳ test en cours…", dim)));
    } else if let Some((_, lignes, ok)) = &st.result {
        let col = if *ok {
            Style::default().fg(palette.green())
        } else {
            Style::default().fg(palette.gold())
        };
        for l in lignes {
            right.push(Line::from(Span::styled(l.clone(), col)));
        }
    } else {
        right.push(Line::from(Span::styled("Entrée → tester ta solution", dim)));
    }
    frame.render_widget(
        Paragraph::new(right)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(dim)
                    .title(Span::styled(format!("  {}  ", e.id), Style::default().fg(palette.accent()))),
            ),
        chunks[1],
    );
}
