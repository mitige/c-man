//! Moteur de quiz : banque embarquée (content/quiz/*.toml) + flashcards
//! générées depuis les pièges des fiches.

use serde::Deserialize;
use std::sync::LazyLock;

// QUIZ_FILES est généré par build.rs dans le module content.
use crate::content::QUIZ_FILES;

#[derive(Debug, Clone, Deserialize)]
pub struct Question {
    pub id: String,
    pub category: String,
    pub difficulty: u8,
    pub question: String,
    pub choices: Vec<String>,
    pub answer: usize,
    pub explanation: String,
    #[serde(default)]
    pub related: Vec<String>,
}

pub static QUESTIONS: LazyLock<Vec<Question>> = LazyLock::new(|| {
    QUIZ_FILES
        .iter()
        .map(|raw| toml::from_str(raw).expect("quiz embarqué invalide"))
        .collect()
});

/// Sélection déterministe-mais-variée : n questions d'une catégorie (ou toutes),
/// mélangées avec une graine temporelle.
pub fn pick(category: Option<&str>, n: usize, seed: u64) -> Vec<&'static Question> {
    let mut pool: Vec<&'static Question> = QUESTIONS
        .iter()
        .filter(|q| category.is_none_or(|c| q.category == c))
        .collect();
    // Fisher-Yates avec LCG (pas de dep rand)
    let mut state = seed.max(1);
    for i in (1..pool.len()).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        pool.swap(i, j);
    }
    pool.truncate(n);
    pool
}

/// Flashcards dérivées des pièges des fiches : "Piège — <fiche>" → la réponse
/// est le piège à reconnaître/éviter.
pub fn flashcards() -> Vec<(&'static str, &'static str)> {
    let mut cards = Vec::new();
    for e in crate::content::ENTRIES.iter() {
        for g in &e.gotchas {
            cards.push((e.id.as_str(), g.as_str()));
        }
    }
    cards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_respects_category_and_count() {
        let qs = pick(Some("memoire"), 3, 42);
        assert!(qs.len() <= 3);
        assert!(qs.iter().all(|q| q.category == "memoire"));
    }

    #[test]
    fn flashcards_nonempty() {
        assert!(!flashcards().is_empty());
    }
}
