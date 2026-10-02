//! Écran Flashcards : révision éclair des pièges classiques, tirés des fiches.

use super::{draw_scroll_panel, ScreenKind, Shell};
use crate::quiz;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

pub(crate) struct FlashState {
    deck: Vec<(&'static str, &'static str)>, // (fiche, piège)
    idx: usize,
    flipped: bool,
    known: u32,
    review: u32,
    done: bool,
}

impl FlashState {
    pub(crate) fn new() -> Self {
        let mut deck = quiz::flashcards();
        // répétition espacée : les cartes « à revoir » d'abord (tri par tier)
        let prog = crate::progress::load();
        deck.sort_by_key(|(fiche, _)| prog.flash_priority(fiche));
        // mélange DANS chaque tier (préserve l'ordre espacé, garde la variété)
        let mut state = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(1)
            .max(1);
        let mut lcg = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize
        };
        // découpe en tiers contigus et mélange chacun
        let mut start = 0;
        while start < deck.len() {
            let tier = prog.flash_priority(deck[start].0).0;
            let mut end = start;
            while end < deck.len() && prog.flash_priority(deck[end].0).0 == tier {
                end += 1;
            }
            // mélange du sous-tableau [start..end]
            for i in (start + 1..end).rev() {
                let j = start + lcg() % (i + 1 - start);
                deck.swap(i, j);
            }
            start = end;
        }
        Self {
            deck,
            idx: 0,
            flipped: false,
            known: 0,
            review: 0,
            done: false,
        }
    }
}

/// Enregistre le résultat d'une carte dans la progression.
fn record(fiche: &str, known: bool) {
    let mut p = crate::progress::load();
    p.record_flash(fiche, known);
    crate::progress::save(&p);
}

pub(crate) fn on_key(shell: &mut Shell, key: KeyEvent) {
    let st = &mut shell.flashcards;
    if st.done {
        match key.code {
            KeyCode::Esc => shell.set_screen(ScreenKind::Docs),
            KeyCode::Char('r') => *st = FlashState::new(),
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Esc => shell.set_screen(ScreenKind::Docs),
        // espace/Entrée : retourner la carte
        KeyCode::Char(' ') | KeyCode::Enter => st.flipped = !st.flipped,
        // 'o' : ouvrir la fiche liée à cette carte (tu ne connais pas le piège)
        KeyCode::Char('o') => {
            if let Some((fiche, _)) = st.deck.get(st.idx) {
                let fiche = fiche.to_string();
                if let Some(idx) = c_man::content::ENTRIES.iter().position(|e| e.id == fiche) {
                    shell.docs.open_entry(idx);
                    shell.set_screen(ScreenKind::Docs);
                }
            }
        }
        // y = je savais, n = à revoir (uniquement une fois retournée)
        KeyCode::Char('y') if st.flipped => {
            st.known += 1;
            record(st.deck[st.idx].0, true);
            st.idx += 1;
            st.flipped = false;
            if st.idx >= st.deck.len() {
                st.done = true;
            }
        }
        KeyCode::Char('n') if st.flipped => {
            st.review += 1;
            record(st.deck[st.idx].0, false);
            st.idx += 1;
            st.flipped = false;
            if st.idx >= st.deck.len() {
                st.done = true;
            }
        }
        _ => {}
    }
}

pub(crate) fn draw(frame: &mut Frame, shell: &mut Shell, area: Rect) {
    let palette = *shell.palette();
    let st = &shell.flashcards;
    let accent = Style::default().fg(palette.accent());
    let dim = Style::default().fg(palette.dim());
    let mut lines: Vec<Line> = Vec::new();

    if st.done {
        let total = st.known + st.review;
        let pct = if total > 0 { st.known * 100 / total } else { 0 };
        lines.push(Line::from(Span::styled(
            "Session terminée",
            accent.add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::default());
        lines.push(Line::from(format!(
            "Connus : {} · À revoir : {} ({} %)",
            st.known, st.review, pct
        )));
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            if pct >= 80 {
                "Solide. Les pièges te connaissent."
            } else {
                "Rejoue demain — la répétition espacée gagne."
            },
            dim,
        )));
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "r : nouveau paquet · Échap : retour",
            dim,
        )));
    } else if let Some((fiche, piege)) = st.deck.get(st.idx) {
        lines.push(Line::from(Span::styled(
            format!("Carte {}/{}", st.idx + 1, st.deck.len()),
            dim,
        )));
        lines.push(Line::default());
        // recto : la fiche, on devine le piège
        lines.push(Line::from(Span::styled(
            format!("« {fiche} »"),
            accent.add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::default());
        if st.flipped {
            // verso : le piège
            lines.push(Line::from(Span::styled(
                "Le piège :",
                Style::default().fg(palette.gold()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::default());
            for l in super::md_to_lines(piege, 96, &palette) {
                lines.push(l);
            }
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "y je savais · n à revoir",
                dim,
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "Quel est le piège classique ici ?",
                dim,
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "espace pour retourner",
                dim,
            )));
        }
    }
    draw_scroll_panel(
        frame,
        area,
        "flashcards — les pièges",
        &lines,
        0,
        palette.accent(),
        "espace retourner · y je savais · n à revoir · Échap docs",
    );
}
