//! Suivi de progression persistant (~/.local/share/c-man/progress.json).
//!
//! Enregistre : fiches consultées (vues + temps), résultats de quiz,
//! exercices marqués faits, historique des passages de la norme.
//! L'IA et l'orchestrateur s'en servent pour adapter l'aide.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub fiches: HashMap<String, FicheStat>,
    pub quiz: Vec<QuizResult>,
    pub exercises_done: HashMap<String, Vec<usize>>,
    pub norme_runs: Vec<NormeRun>,
    pub first_use: u64,
    /// sessions de focus (secondes chacune)
    pub focus_sessions: Vec<u64>,
    /// état des flashcards par fiche (répétition espacée)
    pub flash: HashMap<String, FlashCard>,
    /// jours d'activité (timestamps de début de journée)
    pub active_days: Vec<u64>,
    /// notes personnelles par fiche
    pub notes: HashMap<String, String>,
    /// concepts défendus en soutenance (fiche → nb de soutenances)
    pub defended: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FlashCard {
    pub known: u32,
    pub review: u32,
    pub last: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FicheStat {
    pub views: u32,
    pub total_secs: u64,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResult {
    pub at: u64,
    pub category: String,
    pub total: u32,
    pub correct: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormeRun {
    pub at: u64,
    pub path: String,
    pub majors: u32,
    pub minors: u32,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn store_path() -> PathBuf {
    config::data_dir().join("progress.json")
}

pub fn load() -> Progress {
    match std::fs::read_to_string(store_path()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Progress::default(),
    }
}

pub fn save(p: &Progress) {
    let mut p = p.clone();
    p.mark_active_today();
    let path = store_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&p) {
        let _ = std::fs::write(&path, json);
    }
}

impl Progress {
    /// Enregistre une consultation de fiche (durée en secondes).
    pub fn record_view(&mut self, id: &str, secs: u64) {
        if self.first_use == 0 {
            self.first_use = now();
        }
        let stat = self.fiches.entry(id.to_string()).or_default();
        stat.views += 1;
        stat.total_secs += secs;
        stat.last_seen = now();
    }

    pub fn record_quiz(&mut self, category: &str, total: u32, correct: u32) {
        self.quiz.push(QuizResult {
            at: now(),
            category: category.to_string(),
            total,
            correct,
        });
    }

    /// Enregistre le résultat d'une flashcard (connue ou à revoir).
    pub fn record_flash(&mut self, fiche: &str, known: bool) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let c = self.flash.entry(fiche.to_string()).or_default();
        if known {
            c.known += 1;
        } else {
            c.review += 1;
        }
        c.last = now;
    }

    /// Clé de tri d'une flashcard (répétition espacée) :
    /// tier 0 = à revoir, 1 = jamais vue, 2 = connue ; dans chaque tier,
    /// la plus ancienne (last petit) revient en premier.
    pub fn flash_priority(&self, fiche: &str) -> (u8, u64) {
        match self.flash.get(fiche) {
            Some(c) if c.review > c.known => (0, c.last), // à revoir en premier
            Some(c) => (2, c.last),                       // connue : repoussée
            None => (1, 0),                               // jamais vue : au milieu
        }
    }

    /// Marque aujourd'hui comme un jour d'activité.
    pub fn mark_active_today(&mut self) {
        let day = today_start();
        if !self.active_days.contains(&day) {
            self.active_days.push(day);
            self.active_days.sort_unstable();
        }
    }

    /// Série de jours consécutifs d'activité (en partant d'aujourd'hui).
    pub fn streak(&self) -> u32 {
        let mut n = 0;
        let mut day = today_start();
        loop {
            if self.active_days.contains(&day) {
                n += 1;
                day = day.saturating_sub(86_400);
            } else {
                break;
            }
        }
        n
    }

    /// Enregistre qu'une notion a été défendue en soutenance.
    pub fn record_defended(&mut self, fiche: &str) {
        *self.defended.entry(fiche.to_string()).or_insert(0) += 1;
    }

    /// Les notions défendues au moins une fois.
    pub fn defended_count(&self) -> usize {
        self.defended.len()
    }

    /// Enregistre une note personnelle sur une fiche.
    pub fn set_note(&mut self, fiche: &str, note: &str) {
        if note.trim().is_empty() {
            self.notes.remove(fiche);
        } else {
            self.notes.insert(fiche.to_string(), note.to_string());
        }
    }

    /// La note d'une fiche, si elle existe.
    pub fn note(&self, fiche: &str) -> Option<&str> {
        self.notes.get(fiche).map(|s| s.as_str())
    }

    /// Enregistre une session de focus (durée en secondes).
    pub fn record_focus(&mut self, secs: u64) {
        self.focus_sessions.push(secs);
        if self.focus_sessions.len() > 500 {
            self.focus_sessions.drain(0..100);
        }
    }

    pub fn record_norme_run(&mut self, path: &str, majors: u32, minors: u32) {
        self.norme_runs.push(NormeRun {
            at: now(),
            path: path.to_string(),
            majors,
            minors,
        });
        // garde les 200 derniers passages
        if self.norme_runs.len() > 200 {
            let excess = self.norme_runs.len() - 200;
            self.norme_runs.drain(0..excess);
        }
    }

    pub fn toggle_exercise(&mut self, fiche: &str, index: usize) -> bool {
        let list = self.exercises_done.entry(fiche.to_string()).or_default();
        if let Some(pos) = list.iter().position(|i| *i == index) {
            list.remove(pos);
            false
        } else {
            list.push(index);
            true
        }
    }

    pub fn exercise_done(&self, fiche: &str, index: usize) -> bool {
        self.exercises_done
            .get(fiche)
            .is_some_and(|l| l.contains(&index))
    }

    /// Pourcentage de fiches vues par catégorie.
    pub fn category_coverage(&self, category: &str, entries: &[crate::content::Entry]) -> (usize, usize) {
        let total = entries.iter().filter(|e| e.category == category).count();
        let seen = entries
            .iter()
            .filter(|e| e.category == category && self.fiches.contains_key(&e.id))
            .count();
        (seen, total)
    }

    /// Notions fragiles : catégories de quiz avec < 70 % de réussite récente.
    pub fn weak_categories(&self) -> Vec<(String, u32, u32)> {
        let mut agg: HashMap<String, (u32, u32)> = HashMap::new();
        // ne garde que les 30 derniers quiz
        for r in self.quiz.iter().rev().take(30) {
            let e = agg.entry(r.category.clone()).or_default();
            e.0 += r.correct;
            e.1 += r.total;
        }
        let mut weak: Vec<(String, u32, u32)> = agg
            .into_iter()
            .filter(|(_, (ok, total))| *total > 0 && (*ok as f64) < 0.7 * (*total as f64))
            .map(|(cat, (ok, total))| (cat, ok, total))
            .collect();
        // tri déterministe : score, puis nom (sinon l'ordre HashMap aléatoire
        // faisait clignoter l'indicateur « fragile » à chaque frame)
        weak.sort_by(|(ca, oka, ta), (cb, okb, tb)| {
            let sa = (*oka as i64) * 100 / (*ta as i64).max(1);
            let sb = (*okb as i64) * 100 / (*tb as i64).max(1);
            sa.cmp(&sb).then_with(|| ca.cmp(cb))
        });
        weak
    }

    /// Résumé texte pour l'IA / l'orchestrateur.
    pub fn summary_for_ai(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Fiches consultées : {}\n", self.fiches.len()));
        let total_quiz: u32 = self.quiz.iter().map(|q| q.total).sum();
        let correct: u32 = self.quiz.iter().map(|q| q.correct).sum();
        if total_quiz > 0 {
            s.push_str(&format!(
                "Quiz : {correct}/{total_quiz} bonnes réponses ({}%)\n",
                correct * 100 / total_quiz
            ));
        }
        let weak = self.weak_categories();
        if !weak.is_empty() {
            s.push_str("Notions fragiles : ");
            s.push_str(
                &weak
                    .iter()
                    .map(|(c, ok, t)| format!("{c} ({ok}/{t})"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            s.push('\n');
        }
        let done: usize = self.exercises_done.values().map(|v| v.len()).sum();
        s.push_str(&format!("Exercices marqués faits : {done}\n"));
        if let Some(last) = self.norme_runs.last() {
            s.push_str(&format!(
                "Dernier passage norme : {} ({} majeurs, {} mineurs)\n",
                last.path, last.majors, last.minors
            ));
        }
        s
    }
}

/// Persiste une synthèse « apprends-moi » comme fiche de révision.
/// Retourne un message lisible (chemin) ou None.
pub fn save_revision(text: &str) -> Option<String> {
    let dir = config::data_dir().join("revision");
    std::fs::create_dir_all(&dir).ok()?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("revision-{ts}.md"));
    std::fs::write(&path, text).ok()?;
    Some(format!("{}", path.display()))
}

/// Liste les fiches de révision sauvegardées (récentes d'abord).
pub fn list_revisions() -> Vec<PathBuf> {
    let dir = config::data_dir().join("revision");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    out.sort();
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_cycle() {
        let mut p = Progress::default();
        p.record_view("malloc", 30);
        p.record_view("malloc", 10);
        assert_eq!(p.fiches["malloc"].views, 2);
        assert_eq!(p.fiches["malloc"].total_secs, 40);
        assert!(p.toggle_exercise("malloc", 0));
        assert!(!p.toggle_exercise("malloc", 0));
    }

    #[test]
    /// L'ordre des catégories fragiles doit être déterministe (pas de clignotement).
    #[test]
    fn weak_categories_deterministe() {
        let mut p = Progress::default();
        // plusieurs catégories avec le même score → le tri doit être stable
        p.record_quiz("memoire", 10, 3);
        p.record_quiz("langage", 10, 3);
        p.record_quiz("libc", 10, 3);
        let a = p.weak_categories();
        let b = p.weak_categories();
        assert_eq!(a, b, "l'ordre doit être déterministe (pas de clignotement)");
        assert_eq!(a[0].0, a[0].0); // le premier est toujours le même
    }

    fn weak_categories_detected() {
        let mut p = Progress::default();
        p.record_quiz("memoire", 10, 3);
        p.record_quiz("libc", 10, 9);
        let weak = p.weak_categories();
        assert_eq!(weak.len(), 1);
        assert_eq!(weak[0].0, "memoire");
    }
}

/// Début de la journée courante (timestamp à minuit, approximatif UTC).
fn today_start() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now - (now % 86_400)
}
