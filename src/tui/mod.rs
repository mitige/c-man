//! Shell TUI : héberge le navigateur de docs et tous les écrans outils.
//!
//! Navigation : `Ctrl+O` ouvre le menu outils depuis n'importe où ;
//! `!` lance dsh (handoff terminal, discret) ; `Échap` revient aux docs.

pub(crate) mod docs;
pub(crate) mod exoview;
mod flashcards;
pub(crate) mod quizview;
pub(crate) mod tools;

use c_man::exo;
use c_man::compile::CompileResult;
use c_man::config::Config;
use c_man::content::{by_id, ENTRIES};
use c_man::markdown::{parse, MdLine};
use c_man::norme::Report as NormeReport;
use c_man::progress::Progress;
use c_man::render::wrap_segments;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use docs::{segments_to_spans, DocsApp, Palette};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

// ------------------------------------------------------------------ screens

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScreenKind {
    Docs,
    Norme,
    Compile,
    Quiz,
    Flashcards,
    Exercices,
    Progress,
}

impl ScreenKind {
    fn title(&self) -> &'static str {
        match self {
            Self::Docs => "Documentation",
            Self::Norme => "Norme Epitech",
            Self::Compile => "Compiler & comprendre",
            Self::Quiz => "Quiz",
            Self::Flashcards => "Flashcards",
            Self::Exercices => "Exercices",
            Self::Progress => "Progression",
        }
    }
}

/// Résultats renvoyés par les threads de travail vers la boucle TUI.
pub(crate) enum AsyncMsg {
    Norme(Result<NormeReport, String>),
    Compile(Box<CompileResult>),
    ExoResult((String, Vec<String>, bool)),
}

pub(crate) struct Shell {
    pub(crate) docs: DocsApp,
    pub(crate) screen: ScreenKind,
    pub(crate) menu_open: bool,
    pub(crate) norme: tools::NormeState,
    pub(crate) compile: tools::CompileState,
    pub(crate) quiz: quizview::QuizState,
    pub(crate) flashcards: flashcards::FlashState,
    pub(crate) exo: exoview::ExoState,
    pub(crate) progress_data: Progress,
    pub(crate) config: Config,
    pub(crate) tx: Sender<AsyncMsg>,
    pub(crate) rx: Receiver<AsyncMsg>,
    pub(crate) should_quit: bool,
}

impl Shell {
    fn new(start: Option<usize>) -> Self {
        let (tx, rx) = channel();
        let config = c_man::config::load();
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into());
        Self {
            docs: DocsApp::new(start),
            screen: ScreenKind::Docs,
            menu_open: false,
            norme: tools::NormeState::new(cwd.clone()),
            compile: tools::CompileState::new(cwd),
            quiz: quizview::QuizState::new(),
            flashcards: flashcards::FlashState::new(),
            exo: exoview::ExoState::new(),
            progress_data: c_man::progress::load(),
            config,
            tx,
            rx,
            should_quit: false,
        }
    }

    /// Une saisie texte est-elle active ? (le raccourci global `!` doit alors
    /// rester une lettre comme une autre)
    #[allow(dead_code)]
    fn text_input_active(&self) -> bool {
        if self.menu_open {
            return true;
        }
        match self.screen {
            ScreenKind::Docs => self.docs.focus == docs::Focus::Search,
            ScreenKind::Norme => self.norme.editing_path,
            ScreenKind::Compile => self.compile.editing,
            _ => false,
        }
    }

    /// Colle du texte (bracketed paste) dans la saisie active.
    fn on_paste(&mut self, text: String) {
        let text = text.replace(['\n', '\r'], " ");
        match self.screen {
            ScreenKind::Docs => {
                if self.docs.focus == docs::Focus::Search {
                    self.docs.search.push_str(&text);
                    self.docs.rebuild_rows();
                }
            }
            ScreenKind::Norme => {
                if self.norme.editing_path {
                    self.norme.path.push_str(&text);
                }
            }
            ScreenKind::Compile => {
                if self.compile.editing {
                    self.compile.target.push_str(&text);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn palette(&self) -> &Palette {
        &self.docs.palette
    }

    /// Ouvre une fiche dans l'écran docs (utilisé par les outils).
    /// Tolérant : id exact, sinon meilleure correspondance de recherche.
    pub(crate) fn open_fiche(&mut self, id: &str) {
        let target = by_id(id).map(|e| e.id.clone()).or_else(|| {
            c_man::content::search(id)
                .first()
                .map(|(_, e)| e.id.clone())
        });
        if let Some(id) = target {
            if let Some(idx) = ENTRIES.iter().position(|e| e.id == id) {
                self.docs.current = Some(idx);
                self.docs.scroll = 0;
                self.docs.doc_entry = None; // force rebuild
                self.screen = ScreenKind::Docs;
                self.docs.focus = docs::Focus::Doc;
            }
        }
    }

    fn set_screen(&mut self, s: ScreenKind) {
        self.screen = s;
        self.menu_open = false;
        if s == ScreenKind::Progress {
            self.progress_data = c_man::progress::load();
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        // menu outils
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('o') {
            self.menu_open = !self.menu_open;
            return;
        }
        if self.menu_open {
            self.on_key_menu(key);
            return;
        }
        match self.screen {
            ScreenKind::Docs => {
                // 'e' : ouvrir l'exercice lié à la fiche courante
                if key.code == KeyCode::Char('e')
                    && !key.modifiers.contains(KeyModifiers::CONTROL)
                    && self.docs.focus != docs::Focus::Search
                {
                    if let Some(entry) = self.docs.current_entry() {
                        if exo::EXOS.iter().any(|x| x.fiche == entry.id) {
                            self.set_screen(ScreenKind::Exercices);
                            // sélectionne l'exo lié
                            if let Some(pos) = exo::EXOS.iter().position(|x| x.fiche == entry.id) {
                                self.exo.selected = pos;
                            }
                        }
                    }
                    return;
                }
                self.docs.on_key(key);
                if self.docs.should_quit {
                    self.should_quit = true;
                }
            }
            ScreenKind::Norme => tools::norme_on_key(self, key),
            ScreenKind::Compile => tools::compile_on_key(self, key),
            ScreenKind::Quiz => quizview::on_key(self, key),
            ScreenKind::Flashcards => flashcards::on_key(self, key),
            ScreenKind::Exercices => exoview::on_key(self, key),
            ScreenKind::Progress => tools::progress_on_key(self, key),
        }
    }

    fn on_key_menu(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.menu_open = false,
            KeyCode::Char('1') => self.set_screen(ScreenKind::Docs),
            KeyCode::Char('2') => self.set_screen(ScreenKind::Norme),
            KeyCode::Char('3') => self.set_screen(ScreenKind::Compile),
            KeyCode::Char('4') => self.set_screen(ScreenKind::Quiz),
            KeyCode::Char('5') => self.set_screen(ScreenKind::Progress),
            KeyCode::Char('6') => self.set_screen(ScreenKind::Flashcards),
            KeyCode::Char('7') => self.set_screen(ScreenKind::Exercices),
            _ => {}
        }
    }

    fn drain_async(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMsg::Norme(r) => tools::on_norme_result(self, r),
                AsyncMsg::Compile(r) => tools::on_compile_result(self, *r),
                AsyncMsg::ExoResult(r) => exoview::on_exo_result(self, r),
            }
        }
    }

}

// ------------------------------------------------------------------ helpers de rendu

/// Rendu markdown générique (réponses IA, synthèses) en lignes ratatui.
pub(crate) fn md_to_lines(text: &str, width: usize, palette: &Palette) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let text_w = width.max(20);
    let md = parse(text);
    let mut code_buf: Vec<String> = Vec::new();
    let flush = |lines: &mut Vec<Line<'static>>, buf: &mut Vec<String>| {
        if buf.is_empty() {
            return;
        }
        let code = buf.join("\n");
        let bg = palette.code_bg();
        for line in c_man::highlight::highlight_c(&code) {
            let mut spans: Vec<Span<'static>> =
                vec![Span::styled("  ".to_string(), Style::default().bg(bg))];
            for (style, t) in &line {
                spans.push(Span::styled(
                    t.clone(),
                    Style::default()
                        .fg(Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b))
                        .bg(bg),
                ));
            }
            lines.push(Line::from(spans));
        }
        buf.clear();
    };
    for mdline in &md {
        match mdline {
            MdLine::Code(c) => code_buf.push(c.clone()),
            other => {
                flush(&mut lines, &mut code_buf);
                match other {
                    MdLine::Blank => lines.push(Line::default()),
                    MdLine::Heading(_, t) => lines.push(Line::from(Span::styled(
                        t.clone(),
                        Style::default()
                            .fg(palette.accent())
                            .add_modifier(Modifier::BOLD),
                    ))),
                    MdLine::Text(segs) => {
                        for w in wrap_segments(segs, text_w) {
                            lines.push(Line::from(segments_to_spans(palette, &w)));
                        }
                    }
                    MdLine::Bullet(segs) => {
                        for (i, w) in wrap_segments(segs, text_w.saturating_sub(2)).iter().enumerate() {
                            let mut spans = vec![Span::styled(
                                if i == 0 { "• ".to_string() } else { "  ".to_string() },
                                Style::default().fg(palette.accent()),
                            )];
                            spans.extend(segments_to_spans(palette, w));
                            lines.push(Line::from(spans));
                        }
                    }
                    MdLine::BulletNum(num, segs) => {
                        let marker = format!("{num} ");
                        let indent = " ".repeat(marker.len());
                        for (i, w) in wrap_segments(segs, text_w.saturating_sub(marker.len())).iter().enumerate() {
                            let mut spans = vec![Span::styled(
                                if i == 0 { marker.clone() } else { indent.clone() },
                                Style::default().fg(palette.accent()),
                            )];
                            spans.extend(segments_to_spans(palette, w));
                            lines.push(Line::from(spans));
                        }
                    }
                    MdLine::Quote(segs) => {
                        for w in wrap_segments(segs, text_w.saturating_sub(2)) {
                            let mut spans =
                                vec![Span::styled("▎ ".to_string(), Style::default().fg(palette.dim()))];
                            spans.extend(segments_to_spans(palette, &w));
                            lines.push(Line::from(spans));
                        }
                    }
                    MdLine::Code(_) => {}
                }
            }
        }
    }
    flush(&mut lines, &mut code_buf);
    lines
}

/// Panneau scrollable ineffable : titre discret + filet, contenu qui respire,
/// plus de boîte. La hiérarchie se lit par l'espace et le ton.
pub(crate) fn draw_scroll_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    lines: &[Line],
    scroll: usize,
    _border_color: Color,
    footer_hint: &str,
) -> usize {
    let dim = Style::default().fg(Color::Rgb(134, 134, 139));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // titre
            Constraint::Length(1), // filet
            Constraint::Min(3),    // contenu
            Constraint::Length(1), // hint
        ])
        .split(area);
    // titre discret
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {}", title.to_lowercase()),
            dim.add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );
    // filet
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::Rgb(48, 48, 52)),
        ))),
        chunks[1],
    );
    let inner_h = chunks[2].height as usize;
    let max_scroll = lines.len().saturating_sub(inner_h);
    let start = scroll.min(max_scroll);
    let visible: Vec<Line> = lines
        .iter()
        .skip(start)
        .take(inner_h)
        .cloned()
        .collect();
    frame.render_widget(
        Paragraph::new(visible).block(Block::default().padding(ratatui::widgets::Padding {
            left: 2,
            right: 1,
            top: 0,
            bottom: 0,
        })),
        chunks[2],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {footer_hint}"),
            dim,
        ))),
        chunks[3],
    );
    inner_h
}

/// Ligne d'input générique.
pub(crate) fn draw_input(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    palette: &Palette,
) {
    // ineffable : pas de boîte — libellé discret, saisie, filet fin dessous
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {label}"),
            Style::default().fg(palette.dim()),
        ))),
        chunks[0],
    );
    let content = if focused {
        Line::from(vec![
            Span::raw("  "),
            Span::raw(value.to_string()),
            Span::styled("▌", Style::default().fg(palette.accent())),
        ])
    } else {
        Line::from(Span::styled(format!("  {value}"), Style::default()))
    };
    frame.render_widget(Paragraph::new(content), chunks[1]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::Rgb(48, 48, 52)),
        ))),
        chunks[2],
    );
}

/// Popup centré.
pub(crate) fn centered_popup(frame: &mut Frame, area: Rect, w: u16, h: u16) -> Rect {
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w.min(area.width),
        height: h.min(area.height),
    };
    frame.render_widget(Clear, popup);
    popup
}

// ------------------------------------------------------------------ dessin

fn draw(frame: &mut Frame, shell: &mut Shell) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(5)])
        .split(area);

    // barre de titre
    let weak = shell.progress_data.weak_categories();
    let weak_hint = if weak.is_empty() {
        String::new()
    } else {
        format!("   ⚠ fragile: {}", weak[0].0)
    };
    let title = Line::from(vec![
        Span::styled(
            " c-man ",
            Style::default()
                .fg(Color::Black)
                .bg(shell.palette().accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} ", shell.screen.title()),
            Style::default()
                .fg(shell.palette().accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("Ctrl+O outils{weak_hint}"), Style::default().fg(shell.palette().dim())),
    ]);
    frame.render_widget(Paragraph::new(title), chunks[0]);

    match shell.screen {
        ScreenKind::Docs => docs::draw(frame, &mut shell.docs, chunks[1]),
        ScreenKind::Norme => tools::draw_norme(frame, shell, chunks[1]),
        ScreenKind::Compile => tools::draw_compile(frame, shell, chunks[1]),
        ScreenKind::Quiz => quizview::draw(frame, shell, chunks[1]),
        ScreenKind::Flashcards => flashcards::draw(frame, shell, chunks[1]),
        ScreenKind::Exercices => exoview::draw(frame, shell, chunks[1]),
        ScreenKind::Progress => tools::draw_progress(frame, shell, chunks[1]),
    }

    if shell.menu_open {
        draw_menu(frame, shell, area);
    }

    // saisie du prompt dsh (popup bas)
}

fn draw_menu(frame: &mut Frame, shell: &Shell, area: Rect) {
    let popup = centered_popup(frame, area, 40, 11);
    let entries = [
        ("1", "Documentation"),
        ("2", "Norme Epitech"),
        ("3", "Compiler & comprendre"),
        ("4", "Quiz"),
        ("5", "Progression"),
        ("6", "Flashcards"),
        ("7", "Exercices"),
    ];
    let mut lines: Vec<Line> = vec![Line::default()];
    for (key, name) in entries {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {key}  "),
                Style::default()
                    .fg(shell.palette().accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(name.to_string(), Style::default()),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  Échap fermer",
        Style::default().fg(shell.palette().dim()),
    )));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(shell.palette().accent()))
        .title(Span::styled(
            " Outils ",
            Style::default()
                .fg(shell.palette().accent())
                .add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Régression : un scroll à usize::MAX (arrivée d'une réponse IA) doit
    /// montrer le BAS de la page, pas juste la dernière ligne.
    #[test]
    fn scroll_max_shows_bottom_page() {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let lines: Vec<Line> = (1..=50)
            .map(|i| Line::from(format!("ligne {i}")))
            .collect();
        terminal
            .draw(|f| {
                draw_scroll_panel(
                    f,
                    f.area(),
                    "test",
                    &lines,
                    usize::MAX,
                    Color::Cyan,
                    "",
                );
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        // la dernière ligne est visible…
        assert!(text.contains("ligne 50"), "dernière ligne absente");
        // …et toute la page avec elle (9 lignes visibles : 42..=50), pas juste elle
        assert!(text.contains("ligne 42"), "le bas de page devrait être visible");
        assert!(!text.contains("ligne 41"), "la page ne devrait pas remonter plus haut");
    }
}

// ------------------------------------------------------------------ boucle

/// Init terminal avec bracketed paste (pour que Ctrl+Shift+V colle en bloc).
fn init_terminal() -> io::Result<ratatui::DefaultTerminal> {
    use crossterm::event::EnableBracketedPaste;
    use crossterm::execute;
    use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableBracketedPaste)?;
    ratatui::DefaultTerminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))
}

fn restore_terminal() {
    use crossterm::event::DisableBracketedPaste;
    use crossterm::execute;
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
    let _ = execute!(io::stdout(), DisableBracketedPaste, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

/// Lance la TUI. `start_topic` ouvre directement une fiche si donné.
pub fn run(start_topic: Option<&str>) -> io::Result<()> {
    if ENTRIES.is_empty() {
        eprintln!("c-man: aucune fiche embarquée (content/ vide à la compilation).");
        return Ok(());
    }
    let start = start_topic
        .and_then(by_id)
        .map(|e| ENTRIES.iter().position(|x| x.id == e.id).unwrap());
    let mut shell = Shell::new(start);

    let mut terminal = init_terminal()?;
    let result = run_loop(&mut terminal, &mut shell);
    restore_terminal();
    result
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal, shell: &mut Shell) -> io::Result<()> {
    while !shell.should_quit {
        shell.drain_async();
        terminal.draw(|frame| draw(frame, shell))?;
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => shell.on_key(key),
                Event::Paste(text) => shell.on_paste(text),
                _ => {}
            }
        }
    }
    // sauvegarde la progression en quittant
    c_man::progress::save(&shell.progress_data);
    Ok(())
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Le panneau scrollable : titre en haut, hint en bas, pas de chevauchement.
    #[test]
    fn scroll_panel_layout_no_overlap() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let lines: Vec<Line> = (1..=10).map(|i| Line::from(format!("ligne {i}"))).collect();
        terminal
            .draw(|f| {
                draw_scroll_panel(f, f.area(), "Test Panel", &lines, 0, Color::Cyan, "hint-bas");
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        // titre en haut (row 0)
        let row0: String = (0..80).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(row0.contains("test panel"), "titre en haut: {row0:?}");
        // filet row 1
        let row1: String = (0..80).map(|x| buf[(x, 1)].symbol()).collect();
        assert!(row1.trim().chars().all(|c| c == '─') && !row1.trim().is_empty(), "filet row1: {row1:?}");
        // contenu row 2+
        let row2: String = (0..80).map(|x| buf[(x, 2)].symbol()).collect();
        assert!(row2.contains("ligne 1"), "contenu row2: {row2:?}");
        // hint en bas (row 19)
        let row19: String = (0..80).map(|x| buf[(x, 19)].symbol()).collect();
        assert!(row19.contains("hint-bas"), "hint en bas: {row19:?}");
    }
}
