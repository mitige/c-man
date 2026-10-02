//! Interface terminal interactive (ratatui + crossterm).

use c_man::content::{category_label, Entry, CATEGORIES, ENTRIES};
use c_man::highlight;
use c_man::markdown::{parse, MdLine, Seg};
use c_man::render::wrap_segments;
use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Focus {
    Search,
    List,
    Doc,
}

/// Une ligne visible du panneau de gauche.
#[derive(Clone)]
pub(crate) enum Row {
    Header(&'static str),
    Item(usize), // index dans ENTRIES
}

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    enabled: bool,
}

impl Palette {
    /// Thème « ineffable » — sobriété inspirée d'Apple : un seul accent bleu,
    /// hiérarchie par niveaux de gris, respiration, aucune surcharge visuelle.

    /// Accent unique : bleu système, doux et assuré.
    pub(crate) fn accent(&self) -> Color {
        if self.enabled { Color::Rgb(96, 165, 250) } else { Color::Reset }
    }
    /// Avertissement doux (pièges) : ambre feutré, jamais criard.
    pub(crate) fn gold(&self) -> Color {
        if self.enabled { Color::Rgb(250, 204, 120) } else { Color::Reset }
    }
    /// Validation / exemples : vert désaturé.
    pub(crate) fn green(&self) -> Color {
        if self.enabled { Color::Rgb(134, 199, 152) } else { Color::Reset }
    }
    /// Accent secondaire discret (catégories, liens) : lavande grisée.
    pub(crate) fn purple(&self) -> Color {
        if self.enabled { Color::Rgb(165, 155, 210) } else { Color::Reset }
    }
    /// Texte tertiaire : gris qui s'efface.
    pub(crate) fn dim(&self) -> Color {
        if self.enabled { Color::Rgb(134, 134, 139) } else { Color::Reset }
    }
    /// Code inline : gris très clair (presque blanc), pas de couleur criarde.
    pub(crate) fn code_fg(&self) -> Color {
        if self.enabled { Color::Rgb(235, 235, 240) } else { Color::Reset }
    }
    /// Blocs de code : fond à peine plus sombre que le terminal, sans teinte.
    pub(crate) fn code_bg(&self) -> Color {
        if self.enabled { Color::Rgb(28, 28, 32) } else { Color::Reset }
    }
    /// Sélection : gris-bleu très doux, jamais saturé.
    pub(crate) fn sel_bg(&self) -> Color {
        if self.enabled { Color::Rgb(44, 50, 64) } else { Color::Reset }
    }
}

pub(crate) struct DocsApp {
    pub(crate) focus: Focus,
    pub(crate) search: String,
    pub(crate) rows: Vec<Row>,
    pub(crate) list_state: ListState,
    pub(crate) current: Option<usize>,
    pub(crate) history: Vec<usize>,
    pub(crate) scroll: usize,
    pub(crate) doc_lines: Vec<Line<'static>>,
    pub(crate) doc_width: u16,
    pub(crate) doc_entry: Option<usize>,
    pub(crate) doc_height: usize,
    pub(crate) show_help: bool,
    pub(crate) palette: Palette,
    pub(crate) should_quit: bool,
}

impl DocsApp {
    pub(crate) fn new(start: Option<usize>) -> Self {
        let palette = Palette {
            enabled: std::env::var_os("NO_COLOR").is_none(),
        };
        let mut app = Self {
            focus: Focus::List,
            search: String::new(),
            rows: Vec::new(),
            list_state: ListState::default(),
            current: None,
            history: Vec::new(),
            scroll: 0,
            doc_lines: Vec::new(),
            doc_width: 0,
            doc_entry: None,
            doc_height: 1,
            show_help: false,
            palette,
            should_quit: false,
        };
        app.rebuild_rows();
        if let Some(idx) = start {
            app.open_entry(idx);
        } else {
            let idx = ENTRIES.iter().position(|e| e.id == "bienvenue").unwrap_or(0);
            if !ENTRIES.is_empty() {
                app.current = Some(idx);
            }
            app.select_row_of_current();
        }
        app
    }

    pub(crate) fn rebuild_rows(&mut self) {
        self.rows.clear();
        if !self.search.is_empty() {
            let mut scored: Vec<(i64, usize)> = ENTRIES
                .iter()
                .enumerate()
                .filter_map(|(i, e)| c_man::content::relevance(&self.search, e).map(|s| (s, i)))
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            self.rows = scored.into_iter().map(|(_, i)| Row::Item(i)).collect();
        } else {
            for (cid, label) in CATEGORIES {
                let mut first = true;
                for (i, e) in ENTRIES.iter().enumerate() {
                    if e.category == *cid {
                        if first {
                            self.rows.push(Row::Header(label));
                            first = false;
                        }
                        self.rows.push(Row::Item(i));
                    }
                }
            }
        }
        // sélection valide : premier résultat en recherche, position conservée sinon
        let sel = if self.search.is_empty() {
            self.list_state
                .selected()
                .unwrap_or(0)
                .min(self.rows.len().saturating_sub(1))
        } else {
            0
        };
        self.list_state.select(Some(self.next_item_index(sel, true)));
    }

    pub(crate) fn next_item_index(&self, from: usize, forward: bool) -> usize {
        if self.rows.is_empty() {
            return 0;
        }
        let mut i = from.min(self.rows.len() - 1);
        for _ in 0..self.rows.len() {
            if matches!(self.rows[i], Row::Item(_)) {
                return i;
            }
            i = if forward {
                (i + 1) % self.rows.len()
            } else {
                (i + self.rows.len() - 1) % self.rows.len()
            };
        }
        0
    }

    pub(crate) fn move_selection(&mut self, delta: i64) {
        if self.rows.is_empty() {
            return;
        }
        let len = self.rows.len() as i64;
        let cur = self.list_state.selected().unwrap_or(0) as i64;
        let mut next = (cur + delta).rem_euclid(len) as usize;
        if matches!(self.rows[next], Row::Header(_)) {
            let step = if delta >= 0 { 1 } else { -1 };
            next = (next as i64 + step).rem_euclid(len) as usize;
            if matches!(self.rows[next], Row::Header(_)) {
                return;
            }
        }
        self.list_state.select(Some(next));
        // met à jour l'aperçu si on est en mode liste
        if self.focus != Focus::Doc {
            if let Row::Item(idx) = self.rows[next] {
                if self.current != Some(idx) {
                    self.current = Some(idx);
                    self.scroll = 0;
                }
            }
        }
    }

    pub(crate) fn selected_entry_index(&self) -> Option<usize> {
        let sel = self.list_state.selected()?;
        match self.rows.get(sel) {
            Some(Row::Item(idx)) => Some(*idx),
            _ => None,
        }
    }

    pub(crate) fn open_entry(&mut self, idx: usize) {
        if let Some(cur) = self.current {
            if cur != idx {
                self.history.push(cur);
            }
        }
        self.current = Some(idx);
        self.scroll = 0;
        self.focus = Focus::Doc;
    }

    pub(crate) fn open_selected(&mut self) {
        if let Some(idx) = self.selected_entry_index() {
            self.open_entry(idx);
        }
    }

    pub(crate) fn open_related(&mut self, offset: isize) {
        let Some(cur) = self.current else { return };
        // cycle sur les fiches liées
        let ids: Vec<usize> = ENTRIES[cur]
            .related
            .iter()
            .filter_map(|id| ENTRIES.iter().position(|e| &e.id == id))
            .collect();
        if ids.is_empty() {
            return;
        }
        let next = match ids.iter().position(|i| *i == cur) {
            Some(pos) => (pos as isize + offset).rem_euclid(ids.len() as isize) as usize,
            // la fiche courante n'est pas dans ses propres liens : premier (ou dernier)
            None => {
                if offset >= 0 {
                    0
                } else {
                    ids.len() - 1
                }
            }
        };
        self.open_entry(ids[next]);
    }

    pub(crate) fn go_back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.current = Some(prev);
            self.scroll = 0;
        } else {
            self.focus = Focus::List;
        }
    }

    pub(crate) fn select_row_of_current(&mut self) {
        if let Some(cur) = self.current {
            if let Some(row_idx) = self
                .rows
                .iter()
                .position(|r| matches!(r, Row::Item(i) if *i == cur))
            {
                self.list_state.select(Some(row_idx));
            }
        }
    }

    pub(crate) fn current_entry(&self) -> Option<&'static Entry> {
        self.current.map(|i| &ENTRIES[i])
    }

    pub(crate) fn scroll_doc(&mut self, delta: i64) {
        let max = self
            .doc_lines
            .len()
            .saturating_sub(self.doc_height.max(1));
        let new = (self.scroll as i64 + delta).clamp(0, max as i64);
        self.scroll = new as usize;
    }

    pub(crate) fn rebuild_doc(&mut self, width: u16) {
        // cache invalidé si la fiche OU la largeur change
        if self.doc_width == width && self.doc_entry == self.current && !self.doc_lines.is_empty()
        {
            return;
        }
        self.doc_width = width;
        self.doc_entry = self.current;
        self.doc_lines = match self.current_entry() {
            Some(entry) => build_doc_lines(entry, width as usize, &self.palette),
            None => vec![Line::from("Aucune fiche sélectionnée.")],
        };
        self.scroll_doc(0);
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        if self.show_help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
                self.show_help = false;
            }
            return;
        }
        match self.focus {
            Focus::Search => self.on_key_search(key),
            Focus::List => self.on_key_list(key),
            Focus::Doc => self.on_key_doc(key),
        }
    }

    fn on_key_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search.clear();
                self.focus = Focus::List;
                self.rebuild_rows();
                self.select_row_of_current();
            }
            KeyCode::Enter => {
                self.focus = Focus::List;
                if let Some(idx) = self.selected_entry_index() {
                    self.open_entry(idx);
                }
            }
            KeyCode::Backspace => {
                self.search.pop();
                if self.search.is_empty() {
                    self.focus = Focus::List;
                }
                self.rebuild_rows();
            }
            KeyCode::Char(c) => {
                self.search.push(c);
                self.rebuild_rows();
            }
            KeyCode::Down => self.move_selection(1),
            KeyCode::Up => self.move_selection(-1),
            _ => {}
        }
    }

    fn on_key_list(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('/') => self.focus = Focus::Search,
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.open_selected(),
            KeyCode::Tab => self.focus = Focus::Doc,
            KeyCode::Char(c) if c.is_alphanumeric() || c == '-' => {
                self.search.push(c);
                self.focus = Focus::Search;
                self.rebuild_rows();
            }
            _ => {}
        }
    }

    fn on_key_doc(&mut self, key: KeyEvent) {
        let page = self.doc_height.saturating_sub(2).max(1) as i64;
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('/') => self.focus = Focus::Search,
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                self.focus = Focus::List
            }
            KeyCode::Tab => self.focus = Focus::List,
            KeyCode::Char('j') | KeyCode::Down => self.scroll_doc(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_doc(-1),
            KeyCode::Char('d') | KeyCode::PageDown | KeyCode::Char(' ') => {
                self.scroll_doc(page)
            }
            KeyCode::Char('u') | KeyCode::PageUp => self.scroll_doc(-page),
            KeyCode::Char('g') => self.scroll = 0,
            KeyCode::Char('G') => self.scroll_doc(i64::MAX / 2),
            KeyCode::Char('b') => self.go_back(),
            KeyCode::Char('n') => self.open_related(1),
            KeyCode::Char('p') => self.open_related(-1),
            _ => {}
        }
    }
}

pub(crate) fn seg_style(palette: &Palette, seg: Seg) -> Style {
    match seg {
        Seg::Normal => Style::default(),
        Seg::Bold => Style::default().add_modifier(Modifier::BOLD),
        Seg::Code => Style::default().fg(palette.code_fg()),
        Seg::Dim => Style::default().fg(palette.dim()),
    }
}

pub(crate) fn segments_to_spans<'a>(palette: &Palette, segs: &[(Seg, String)]) -> Vec<Span<'a>> {
    segs.iter()
        .map(|(seg, text)| Span::styled(text.clone(), seg_style(palette, *seg)))
        .collect()
}

/// Construit toutes les lignes du document, pré-wrappées à `width`.
pub(crate) fn build_doc_lines(entry: &Entry, width: usize, palette: &Palette) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let width = width.max(30);
    let accent = Style::default().fg(palette.accent());
    let gold = Style::default().fg(palette.gold());
    let green = Style::default().fg(palette.green());
    let dim = Style::default().fg(palette.dim());

    // Titre
    lines.push(Line::from(Span::styled(
        entry.title.clone(),
        Style::default()
            .fg(palette.accent())
            .add_modifier(Modifier::BOLD),
    )));
    let mut meta = category_label(&entry.category).to_string();
    if let Some(day) = &entry.piscine_day {
        meta.push_str(&format!(" · piscine {day}"));
    }
    if !entry.tags.is_empty() {
        meta.push_str(&format!(" · #{}", entry.tags.join(" #")));
    }
    lines.push(Line::from(Span::styled(meta, dim)));
    lines.push(Line::default());

    // Synopsis encadré
    push_box(
        &mut lines,
        "SYNOPSIS",
        accent,
        entry
            .synopsis
            .lines()
            .map(|l| {
                vec![Span::styled(
                    l.to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                )]
            })
            .collect(),
        width,
    );
    lines.push(Line::default());

    // Description
    let md = parse(&entry.description);
    let text_w = width.saturating_sub(2);
    let mut in_code_block = false;
    let mut code_buf: Vec<String> = Vec::new();
    let flush_code = |lines: &mut Vec<Line<'static>>, buf: &mut Vec<String>| {
        if buf.is_empty() {
            return;
        }
        let code = buf.join("\n");
        let bg = palette.code_bg();
        for line in highlight::highlight_c(&code) {
            let mut spans: Vec<Span<'static>> = vec![Span::styled("  ".to_string(), Style::default().bg(bg))];
            let mut visible = 0usize;
            for (style, text) in &line {
                visible += UnicodeWidthStr::width(text.as_str());
                spans.push(Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b))
                        .bg(bg),
                ));
            }
            let pad = width.saturating_sub(visible + 2);
            if pad > 0 {
                spans.push(Span::styled(" ".repeat(pad), Style::default().bg(bg)));
            }
            lines.push(Line::from(spans));
        }
        buf.clear();
    };
    for mdline in &md {
        match mdline {
            MdLine::Code(c) => {
                in_code_block = true;
                code_buf.push(c.clone());
            }
            other => {
                if in_code_block {
                    flush_code(&mut lines, &mut code_buf);
                    in_code_block = false;
                }
                match other {
                    MdLine::Blank => lines.push(Line::default()),
                    MdLine::Heading(level, text) => {
                        let style = match level {
                            1 => Style::default()
                                .fg(palette.purple())
                                .add_modifier(Modifier::BOLD),
                            2 => Style::default()
                                .fg(palette.accent())
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                            _ => Style::default()
                                .fg(palette.dim())
                                .add_modifier(Modifier::BOLD),
                        };
                        lines.push(Line::from(Span::styled(text.clone(), style)));
                    }
                    MdLine::Text(segs) => {
                        for wrapped in wrap_segments(segs, text_w) {
                            lines.push(Line::from(segments_to_spans(palette, &wrapped)));
                        }
                    }
                    MdLine::Bullet(segs) => {
                        let wrapped = wrap_segments(segs, text_w.saturating_sub(2));
                        for (i, wl) in wrapped.iter().enumerate() {
                            let mut spans = vec![Span::styled(
                                if i == 0 { "• ".to_string() } else { "  ".to_string() },
                                accent,
                            )];
                            spans.extend(segments_to_spans(palette, wl));
                            lines.push(Line::from(spans));
                        }
                    }
                    MdLine::BulletNum(num, segs) => {
                        let marker = format!("{num} ");
                        let indent = " ".repeat(marker.len());
                        let wrapped = wrap_segments(segs, text_w.saturating_sub(marker.len()));
                        for (i, wl) in wrapped.iter().enumerate() {
                            let mut spans = vec![Span::styled(
                                if i == 0 { marker.clone() } else { indent.clone() },
                                accent,
                            )];
                            spans.extend(segments_to_spans(palette, wl));
                            lines.push(Line::from(spans));
                        }
                    }
                    MdLine::Quote(segs) => {
                        for wrapped in wrap_segments(segs, text_w.saturating_sub(2)) {
                            let mut spans = vec![Span::styled("▎ ".to_string(), dim)];
                            spans.extend(segments_to_spans(palette, &wrapped));
                            lines.push(Line::from(spans));
                        }
                    }
                    MdLine::Code(_) => {}
                }
            }
        }
    }
    if in_code_block {
        flush_code(&mut lines, &mut code_buf);
    }
    lines.push(Line::default());

    // Exemple encadré (coloration syntect)
    let bg = palette.code_bg();
    let mut example_lines: Vec<Vec<Span<'static>>> = Vec::new();
    for line in highlight::highlight_c(&entry.example) {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for (style, text) in &line {
            spans.push(Span::styled(
                text.clone(),
                Style::default()
                    .fg(Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b))
                    .bg(bg),
            ));
        }
        example_lines.push(spans);
    }
    push_box(&mut lines, "EXEMPLE", green, example_lines, width);
    lines.push(Line::default());

    // Pièges
    if !entry.gotchas.is_empty() {
        let inner = width.saturating_sub(8);
        let mut items: Vec<Vec<Span<'static>>> = Vec::new();
        for g in &entry.gotchas {
            let segs = parse_inline_for_doc(g);
            let wrapped = wrap_segments(&segs, inner);
            for (i, wl) in wrapped.iter().enumerate() {
                let mut spans = vec![Span::styled(
                    if i == 0 { "⚠ ".to_string() } else { "  ".to_string() },
                    gold,
                )];
                spans.extend(segments_to_spans(palette, wl));
                items.push(spans);
            }
        }
        push_box(&mut lines, "PIÈGES", gold, items, width);
        lines.push(Line::default());
    }

    // Exercices
    if !entry.exercises.is_empty() {
        let inner = width.saturating_sub(9);
        let mut items: Vec<Vec<Span<'static>>> = Vec::new();
        for (i, e) in entry.exercises.iter().enumerate() {
            let segs = parse_inline_for_doc(e);
            let wrapped = wrap_segments(&segs, inner);
            for (j, wl) in wrapped.iter().enumerate() {
                let mut spans = vec![Span::styled(
                    if j == 0 { format!("{}. ", i + 1) } else { "   ".to_string() },
                    green,
                )];
                spans.extend(segments_to_spans(palette, wl));
                items.push(spans);
            }
        }
        push_box(&mut lines, "EXERCICES", green, items, width);
        lines.push(Line::default());
    }

    // Voir aussi
    if !entry.related.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("→ Voir aussi : ", Style::default().fg(palette.purple())),
            Span::styled(entry.related.join(" · "), Style::default().fg(palette.accent())),
            Span::styled("   (n/p pour naviguer)", dim),
        ]));
    }
    lines
}

fn parse_inline_for_doc(text: &str) -> Vec<(Seg, String)> {
    c_man::markdown::parse_inline(text)
}

/// Encadre des lignes de contenu dans une boîte unicode.
/// Section ineffable : un libellé discret, le contenu indenté, un souffle.
/// (Plus de boîtes lourdes — la hiérarchie se lit par l'espace et le ton.)
fn push_box(
    lines: &mut Vec<Line<'static>>,
    title: &str,
    border_style: Style,
    content: Vec<Vec<Span<'static>>>,
    _width: usize,
) {
    let dim = Style::default().fg(Color::Rgb(134, 134, 139));
    lines.push(Line::from(Span::styled(
        title.to_lowercase(),
        dim.add_modifier(Modifier::BOLD),
    )));
    let _ = border_style; // la couleur de section vit dans le contenu, pas dans un cadre
    for spans in content {
        let mut line_spans = vec![Span::raw("  ".to_string())];
        line_spans.extend(spans);
        lines.push(Line::from(line_spans));
    }
    lines.push(Line::default());
}

// ------------------------------------------------------------------ affichage

pub(crate) fn draw(frame: &mut Frame, app: &mut DocsApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    draw_search(frame, app, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(30)])
        .split(chunks[1]);
    draw_list(frame, app, body[0]);
    draw_doc(frame, app, body[1]);
    draw_status(frame, app, chunks[2]);

    if app.show_help {
        draw_help(frame, app, area);
    }
}

pub(crate) fn draw_search(frame: &mut Frame, app: &DocsApp, area: Rect) {
    // ineffable : pas de boîte — une ligne de saisie, un filet discret dessous
    let focused = app.focus == Focus::Search;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let content = if app.search.is_empty() && !focused {
        Line::from(Span::styled(
            "  / pour chercher",
            Style::default().fg(app.palette.dim()),
        ))
    } else {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(app.search.clone(), Style::default()),
            if focused {
                Span::styled("▌", Style::default().fg(app.palette.accent()))
            } else {
                Span::default()
            },
        ])
    };
    frame.render_widget(Paragraph::new(content), chunks[0]);
    let filet = Style::default().fg(app.palette.dim());
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat(area.width as usize), filet))),
        chunks[1],
    );
}

pub(crate) fn draw_list(frame: &mut Frame, app: &mut DocsApp, area: Rect) {
    // ineffable : pas de cadre — une marge, des têtes de catégorie feutrées,
    // la sélection comme un nappage doux. Un filet vertical sépare du contenu.
    let prog = crate::progress::load();
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            Row::Header(label) => ListItem::new(Line::from(Span::styled(
                format!(" {label}"),
                Style::default()
                    .fg(app.palette.dim())
                    .add_modifier(Modifier::BOLD),
            ))),
            Row::Item(idx) => {
                let e = &ENTRIES[*idx];
                let title = if app.search.is_empty() {
                    e.id.clone()
                } else {
                    format!("{} ({})", e.id, category_label(&e.category))
                };
                // indicateur discret de progression : ✓ défendue, · lue
                let (mark, mark_style) = if prog.defended.contains_key(&e.id) {
                    ("✓ ", Style::default().fg(app.palette.green()))
                } else if prog.fiches.contains_key(&e.id) {
                    ("· ", Style::default().fg(app.palette.dim()))
                } else {
                    ("  ", Style::default())
                };
                ListItem::new(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(mark, mark_style),
                    Span::styled(title, Style::default().fg(Color::Rgb(245, 245, 247))),
                ]))
            }
        })
        .collect();
    let highlight_style = if app.focus == Focus::List {
        Style::default().bg(app.palette.sel_bg()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(app.palette.sel_bg())
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::Rgb(58, 58, 62))),
        )
        .highlight_style(highlight_style)
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

pub(crate) fn draw_doc(frame: &mut Frame, app: &mut DocsApp, area: Rect) {
    // ineffable : le contenu respire, sans cadre. Le titre vit dans la marge.
    let inner_h = area.height.saturating_sub(1) as usize;
    app.doc_height = inner_h;
    app.rebuild_doc(area.width.saturating_sub(3));

    let start = app.scroll.min(app.doc_lines.len());
    let visible: Vec<Line> = app
        .doc_lines
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
        area,
    );

    // scrollbar fine, à peine là
    if app.doc_lines.len() > inner_h {
        let mut state = ScrollbarState::new(app.doc_lines.len().saturating_sub(inner_h))
            .position(app.scroll);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(Style::default().fg(Color::Rgb(72, 72, 76)))
                .track_style(Style::default().fg(Color::Rgb(32, 32, 34))),
            area,
            &mut state,
        );
    }
}

pub(crate) fn draw_status(frame: &mut Frame, app: &DocsApp, area: Rect) {
    // ineffable : le strict nécessaire, en gris
    let hints: &str = match app.focus {
        Focus::Search => "Enter ouvrir · Échap annuler",
        Focus::List => "j/k naviguer · Enter ouvrir · Ctrl+O outils · ? aide",
        Focus::Doc => "j/k défiler · espace page · h retour · n/p liens · ? aide",
    };
    let pos = match app.current_entry() {
        Some(_) if app.focus == Focus::Doc => format!(
            "  {}/{}",
            app.scroll + 1,
            app.doc_lines.len().max(1)
        ),
        _ => String::new(),
    };
    let line = Line::from(vec![
        Span::styled(format!("  {hints}"), Style::default().fg(app.palette.dim())),
        Span::styled(pos, Style::default().fg(app.palette.dim())),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

pub(crate) fn draw_help(frame: &mut Frame, app: &DocsApp, area: Rect) {
    let w = area.width.min(64);
    let h = area.height.min(22);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    frame.render_widget(Clear, popup);
    let help = vec![
        ("j / k / ↑ / ↓", "naviguer / défiler"),
        ("espace / PgDn", "page suivante (doc)"),
        ("g / G", "haut / bas du document"),
        ("Enter / l", "ouvrir la fiche"),
        ("h / Esc / ⌫", "retour à la liste"),
        ("/", "recherche floue"),
        ("n / p", "fiche liée suivante / précédente"),
        ("b", "retour historique"),
        ("Tab", "basculer liste ↔ document"),
        ("Ctrl+O", "menu outils (norme, compile, quiz, IA…)"),
        ("!", "lancer dsh (DeepSeek-Harness)"),
        ("?", "cette aide"),
        ("q / Ctrl+C", "quitter"),
    ];
    let mut lines: Vec<Line> = vec![Line::default()];
    for (key, desc) in help {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {key:<18}"),
                Style::default().fg(app.palette.accent()).add_modifier(Modifier::BOLD),
            ),
            Span::raw(desc),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  c-man — ta doc C de piscine 🏊",
        Style::default().fg(app.palette.dim()),
    )));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(app.palette.accent()))
        .title(Span::styled(" Aide ", Style::default().fg(app.palette.accent()).add_modifier(Modifier::BOLD)));
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    fn entry_index(id: &str) -> usize {
        ENTRIES.iter().position(|e| e.id == id).expect(id)
    }

    #[test]
    fn startup_opens_bienvenue() {
        let app = DocsApp::new(None);
        assert_eq!(app.current, Some(entry_index("bienvenue")));
    }

    #[test]
    fn search_malloc_then_enter_opens_malloc() {
        let mut app = DocsApp::new(None);
        app.on_key(key('/'));
        for c in "malloc".chars() {
            app.on_key(key(c));
        }
        assert!(!app.rows.is_empty());
        // le premier résultat doit etre malloc
        let first = app.selected_entry_index().expect("selection");
        assert_eq!(ENTRIES[first].id, "malloc");
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.current, Some(entry_index("malloc")));
        assert_eq!(app.focus, Focus::Doc);
    }

    #[test]
    fn list_navigation_skips_headers() {
        let mut app = DocsApp::new(None);
        app.on_key(key('j'));
        let idx = app.list_state.selected().unwrap();
        assert!(matches!(app.rows[idx], Row::Item(_)));
    }

    #[test]
    fn back_after_related_jump_returns() {
        let mut app = DocsApp::new(None);
        let first = app.current.unwrap();
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.focus, Focus::Doc);
        // saute vers une fiche liée : l'historique doit permettre de revenir
        app.on_key(key('n'));
        let second = app.current.unwrap();
        assert_ne!(first, second);
        app.on_key(key('b'));
        assert_eq!(app.current, Some(first));
    }

    #[test]
    fn list_navigation_updates_preview() {
        let mut app = DocsApp::new(None);
        let first = app.current.unwrap();
        app.on_key(key('j'));
        let previewed = app.current.unwrap();
        assert_ne!(first, previewed);
    }

    #[test]
    fn quit_on_q() {
        let mut app = DocsApp::new(None);
        app.on_key(key('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn render_frame_smoke() {
        let backend = TestBackend::new(110, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = DocsApp::new(None);
        terminal.draw(|f| draw(f, &mut app, f.area())).unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        // design ineffable : plus de titre « Fiches », sections en minuscules
        assert!(text.contains("bienvenue"));
        assert!(text.contains("synopsis"));
    }

    #[test]
    fn navigating_renders_new_doc() {
        // régression : le cache du doc ne tenait compte que de la largeur,
        // la fiche affichée restait figée sur celle du démarrage
        let backend = TestBackend::new(110, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = DocsApp::new(None);
        terminal.draw(|f| draw(f, &mut app, f.area())).unwrap(); // construit bienvenue
        app.on_key(key('j')); // aperçu de la fiche suivante
        terminal.draw(|f| draw(f, &mut app, f.area())).unwrap();
        let expected_id = app.current_entry().unwrap().id.clone();
        assert_ne!(expected_id, "bienvenue");
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(
            text.contains(&format!(" {expected_id} ")),
            "le panneau doc devrait afficher la fiche {expected_id}"
        );
    }

    #[test]
    fn render_search_malloc_frame() {
        let backend = TestBackend::new(110, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = DocsApp::new(None);
        app.on_key(key('/'));
        for c in "malloc".chars() {
            app.on_key(key(c));
        }
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        terminal.draw(|f| draw(f, &mut app, f.area())).unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("malloc"), "le doc malloc devrait etre visible");
        assert!(text.contains("synopsis"), "la section synopsis devrait etre visible");
        // saute en bas de page : les sections finales doivent apparaitre
        app.on_key(key('G'));
        terminal.draw(|f| draw(f, &mut app, f.area())).unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(
            text.contains("EXERCICES") || text.contains("Voir aussi"),
            "les sections de fin devraient etre visibles apres G"
        );
    }
}


#[cfg(test)]
mod dbg_tests {
    use super::*;

    #[test]
    fn debug_doc_top_lines() {
        let entry = c_man::content::by_id("malloc").unwrap();
        let palette = Palette { enabled: false };
        let lines = build_doc_lines(entry, 76, &palette);
        for (i, l) in lines.iter().take(10).enumerate() {
            let text: String = l.spans.iter().map(|s| s.content.as_ref()).collect();
            eprintln!("{i}: {text:?}");
        }
    }
}
