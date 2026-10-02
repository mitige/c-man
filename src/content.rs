//! Chargement des fiches embarquées à la compilation.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/content.rs"));

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub difficulty: u8,
    #[serde(default)]
    pub piscine_day: Option<String>,
    pub synopsis: String,
    pub description: String,
    pub example: String,
    #[serde(default)]
    pub gotchas: Vec<String>,
    #[serde(default)]
    pub exercises: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
}

/// (id, label affiché)
pub const CATEGORIES: &[(&str, &str)] = &[
    ("demarrer", "Démarrer"),
    ("langage", "Langage C"),
    ("memoire", "Mémoire & pointeurs"),
    ("libc", "Libc"),
    ("outils", "Compilation & outils"),
    ("piscine", "Survie piscine"),
    ("bash", "Bash & Unix"),
    ("web", "Web (HTML/CSS/JS)"),
];

pub fn category_label(id: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(cid, _)| *cid == id)
        .map(|(_, label)| *label)
        .unwrap_or("Autres")
}

/// Toutes les fiches, triées par catégorie puis difficulté puis titre.
pub static ENTRIES: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    let mut entries: Vec<Entry> = CONTENT_FILES
        .iter()
        .map(|raw| toml::from_str(raw).expect("fiche embarquée invalide"))
        .collect();
    entries.sort_by(|a, b| {
        let ca = CATEGORIES
            .iter()
            .position(|(c, _)| *c == a.category)
            .unwrap_or(usize::MAX);
        let cb = CATEGORIES
            .iter()
            .position(|(c, _)| *c == b.category)
            .unwrap_or(usize::MAX);
        ca.cmp(&cb)
            .then(a.difficulty.cmp(&b.difficulty))
            .then(a.title.cmp(&b.title))
    });
    entries
});

pub fn by_id(id: &str) -> Option<&'static Entry> {
    let needle = id.to_lowercase();
    ENTRIES.iter().find(|e| e.id.to_lowercase() == needle)
}

/// Score de pertinence d'une fiche pour une requête.
/// Priorité : id exact > préfixe d'id > sous-chaîne d'id > titre > tag > fuzzy.
pub fn relevance(query: &str, entry: &Entry) -> Option<i64> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }
    let id = entry.id.to_lowercase();
    let title = entry.title.to_lowercase();
    if id == q {
        return Some(100_000);
    }
    if id.starts_with(&q) {
        return Some(50_000 - q.len() as i64);
    }
    if let Some(pos) = id.find(&q) {
        return Some(30_000 - pos as i64);
    }
    if title.starts_with(&q) {
        return Some(20_000);
    }
    if let Some(pos) = title.find(&q) {
        return Some(10_000 - pos as i64);
    }
    if entry.tags.iter().any(|t| t.to_lowercase().contains(&q)) {
        return Some(5_000);
    }
    let matcher = SkimMatcherV2::default();
    let hay = format!("{} {} {}", entry.id, entry.title, entry.tags.join(" "));
    matcher
        .fuzzy_match(&hay, &q)
        .filter(|s| *s > 0)
        .map(|s| s.min(1_000))
}

/// Recherche classée : toutes les fiches pertinentes, meilleures d'abord.
pub fn search(query: &str) -> Vec<(i64, &'static Entry)> {
    let mut scored: Vec<(i64, &Entry)> = ENTRIES
        .iter()
        .filter_map(|e| relevance(query, e).map(|s| (s, e)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.id.cmp(&b.1.id)));
    scored
}
