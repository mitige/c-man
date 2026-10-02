//! Écran Quiz : choix de catégorie → questions → score enregistré.

use super::{draw_scroll_panel, ScreenKind, Shell};
use c_man::quiz::{self, Question};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

const CATEGORIES: [(&str, &str); 8] = [
    ("tout", "Tout mélanger"),
    ("langage", "Langage C"),
    ("memoire", "Mémoire & pointeurs"),
    ("libc", "Libc"),
    ("outils", "Compilation & outils"),
    ("piscine", "Survie piscine"),
    ("bash", "Bash & Unix"),
    ("web", "Web"),
];

#[derive(PartialEq)]
pub(crate) enum QuizPhase {
    PickCategory,
    Question,
    Feedback,
    Done,
}

pub(crate) struct QuizState {
    pub(crate) phase: QuizPhase,
    pub(crate) cat_index: usize,
    pub(crate) questions: Vec<&'static Question>,
    pub(crate) current: usize,
    pub(crate) correct: u32,
    pub(crate) picked: Option<usize>,
    pub(crate) category: String,
}

impl QuizState {
    pub(crate) fn new() -> Self {
        Self {
            phase: QuizPhase::PickCategory,
            cat_index: 0,
            questions: Vec::new(),
            current: 0,
            correct: 0,
            picked: None,
            category: String::new(),
        }
    }

    fn start(&mut self) {
        let (key, _) = CATEGORIES[self.cat_index];
        self.category = key.to_string();
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(1);
        self.questions = quiz::pick(if key == "tout" { None } else { Some(key) }, 5, seed);
        self.current = 0;
        self.correct = 0;
        self.picked = None;
        self.phase = if self.questions.is_empty() {
            QuizPhase::Done
        } else {
            QuizPhase::Question
        };
    }
}

pub(crate) fn on_key(shell: &mut Shell, key: KeyEvent) {
    let st = &mut shell.quiz;
    match st.phase {
        QuizPhase::PickCategory => match key.code {
            KeyCode::Esc => shell.set_screen(ScreenKind::Docs),
            KeyCode::Char('j') | KeyCode::Down => {
                st.cat_index = (st.cat_index + 1).min(CATEGORIES.len() - 1)
            }
            KeyCode::Char('k') | KeyCode::Up => st.cat_index = st.cat_index.saturating_sub(1),
            KeyCode::Enter => st.start(),
            _ => {}
        },
        QuizPhase::Question => match key.code {
            KeyCode::Esc => shell.set_screen(ScreenKind::Docs),
            KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                let pick = (c as usize) - ('1' as usize);
                if let Some(q) = st.questions.get(st.current) {
                    if pick < q.choices.len() {
                        st.picked = Some(pick);
                        if pick == q.answer {
                            st.correct += 1;
                        }
                        st.phase = QuizPhase::Feedback;
                    }
                }
            }
            _ => {}
        },
        QuizPhase::Feedback => match key.code {
            KeyCode::Esc => shell.set_screen(ScreenKind::Docs),
            KeyCode::Enter | KeyCode::Char(' ') => {
                st.current += 1;
                st.picked = None;
                if st.current >= st.questions.len() {
                    st.phase = QuizPhase::Done;
                    // enregistre le score
                    shell.progress_data.record_quiz(
                        &st.category,
                        st.questions.len() as u32,
                        st.correct,
                    );
                    c_man::progress::save(&shell.progress_data);
                } else {
                    st.phase = QuizPhase::Question;
                }
            }
            _ => {}
        },
        QuizPhase::Done => match key.code {
            KeyCode::Esc => shell.set_screen(ScreenKind::Docs),
            KeyCode::Char('r') => st.phase = QuizPhase::PickCategory,
            _ => {}
        },
    }
}

pub(crate) fn draw(frame: &mut Frame, shell: &mut Shell, area: Rect) {
    let palette = *shell.palette();
    let st = &shell.quiz;
    let accent = Style::default().fg(palette.accent());
    let dim = Style::default().fg(palette.dim());
    let green = Style::default().fg(palette.green());
    let gold = Style::default().fg(palette.gold());
    let mut lines: Vec<Line> = Vec::new();

    match st.phase {
        QuizPhase::PickCategory => {
            lines.push(Line::from(Span::styled(
                "Choisis ton terrain d'entraînement :",
                accent.add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::default());
            for (i, (_, label)) in CATEGORIES.iter().enumerate() {
                let marker = if i == st.cat_index { "▸" } else { " " };
                let style = if i == st.cat_index {
                    accent.add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let count = quiz::QUESTIONS
                    .iter()
                    .filter(|q| CATEGORIES[i].0 == "tout" || q.category == CATEGORIES[i].0)
                    .count();
                lines.push(Line::from(vec![
                    Span::styled(format!("{marker} "), style),
                    Span::styled(format!("{label:<24}"), style),
                    Span::styled(format!("{count} questions"), dim),
                ]));
            }
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "5 questions par session — court et régulier bat long et rare.",
                dim,
            )));
        }
        QuizPhase::Question | QuizPhase::Feedback => {
            let q = st.questions[st.current];
            lines.push(Line::from(Span::styled(
                format!("Question {}/{}", st.current + 1, st.questions.len()),
                accent.add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::default());
            for l in super::md_to_lines(&q.question, 100, &palette) {
                lines.push(l);
            }
            lines.push(Line::default());
            for (i, choice) in q.choices.iter().enumerate() {
                let mut style = Style::default();
                let mut prefix = format!("  {}. ", i + 1);
                if st.phase == QuizPhase::Feedback {
                    if i == q.answer {
                        style = green.add_modifier(Modifier::BOLD);
                        prefix = format!("  ✓ {}. ", i + 1);
                    } else if st.picked == Some(i) {
                        style = gold;
                        prefix = format!("  ✗ {}. ", i + 1);
                    }
                }
                lines.push(Line::from(Span::styled(format!("{prefix}{choice}"), style)));
            }
            if st.phase == QuizPhase::Feedback {
                lines.push(Line::default());
                let ok = st.picked == Some(q.answer);
                lines.push(Line::from(Span::styled(
                    if ok { "✓ Correct !" } else { "✗ Raté." },
                    if ok { green } else { gold }.add_modifier(Modifier::BOLD),
                )));
                for l in super::md_to_lines(&q.explanation, 100, &palette) {
                    lines.push(l);
                }
                if !q.related.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("→ Relire : ", Style::default().fg(palette.purple())),
                        Span::styled(q.related.join(" · "), accent),
                    ]));
                }
            }
        }
        QuizPhase::Done => {
            let total = st.questions.len() as u32;
            let pct = if total > 0 { st.correct * 100 / total } else { 0 };
            lines.push(Line::from(Span::styled(
                format!("══ Score : {}/{} ({} %) ══", st.correct, total, pct),
                accent.add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::default());
            let verdict = match pct {
                90..=100 => "Solide. Ce thème est validé pour la piscine.",
                70..=89 => "Bien ! Encore deux ou trois fiches à relire.",
                50..=69 => "Mitigé : relis les fiches liées puis retente.",
                _ => "Retourne aux fiches du thème, puis retente le quiz.",
            };
            lines.push(Line::from(verdict));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "r : nouveau quiz · Échap : retour aux docs",
                dim,
            )));
        }
    }
    draw_scroll_panel(frame, area, "quiz", &lines, 0, palette.accent(), "");
}
