//! Contexte projet : donne aux agents IA une vision du dossier où c-man
//! a été lancé (arborescence + contenu des fichiers C, borné).

use std::path::{Path, PathBuf};

/// Nombre max de fichiers lus.
const MAX_FILES: usize = 12;
/// Taille max par fichier (caractères).
const MAX_PER_FILE: usize = 4000;
/// Taille totale max du contexte (caractères).
const MAX_TOTAL: usize = 24_000;

#[derive(Debug, Default)]
pub struct ProjectSnapshot {
    pub cwd: String,
    /// (chemin relatif, contenu éventuellement tronqué)
    pub files: Vec<(String, String)>,
    /// fichiers listés mais non lus (budget atteint)
    pub skipped: Vec<String>,
}

fn is_source(name: &str) -> bool {
    name.ends_with(".c")
        || name.ends_with(".h")
        || name.eq_ignore_ascii_case("makefile")
        || name.ends_with(".toml")
}

fn collect(dir: &Path, base: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 2 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        if p.is_dir() {
            collect(&p, base, depth + 1, out);
        } else if p.is_file() && is_source(&name) {
            out.push(p);
        }
        if out.len() >= MAX_FILES * 3 {
            return;
        }
    }
}

/// Lit le projet (borné) pour injection dans les prompts IA.
pub fn snapshot(cwd: &Path) -> ProjectSnapshot {
    let mut snap = ProjectSnapshot {
        cwd: cwd.to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut paths = Vec::new();
    collect(cwd, cwd, 0, &mut paths);
    let mut total = 0usize;
    for p in paths {
        let rel = p
            .strip_prefix(cwd)
            .map(|r| r.to_string_lossy().to_string())
            .unwrap_or_else(|_| p.display().to_string());
        if snap.files.len() >= MAX_FILES || total >= MAX_TOTAL {
            snap.skipped.push(rel);
            continue;
        }
        match std::fs::read_to_string(&p) {
            Ok(content) => {
                let budget = MAX_PER_FILE.min(MAX_TOTAL - total);
                let truncated = content.len() > budget;
                let text: String = content.chars().take(budget).collect();
                let text = if truncated {
                    format!("{text}\n/* … tronqué … */")
                } else {
                    text
                };
                total += text.len();
                snap.files.push((rel, text));
            }
            Err(_) => snap.skipped.push(rel),
        }
    }
    snap
}

impl ProjectSnapshot {
    /// Texte prêt à injecter dans un prompt.
    pub fn to_prompt(&self) -> String {
        if self.files.is_empty() && self.skipped.is_empty() {
            return format!("Répertoire courant : {} (aucun fichier C détecté)", self.cwd);
        }
        let mut s = format!(
            "PROJET DE L'ÉTUDIANT — répertoire : {}\nFichiers fournis :\n",
            self.cwd
        );
        for (path, content) in &self.files {
            s.push_str(&format!("\n══ {path} ══\n{content}\n"));
        }
        if !self.skipped.is_empty() {
            s.push_str(&format!(
                "\n(non lus, budget atteint : {})\n",
                self.skipped.join(", ")
            ));
        }
        s
    }
}

/// Compte les fichiers source du projet (léger, sans lire le contenu).
pub fn count_sources(cwd: &Path) -> usize {
    let mut paths = Vec::new();
    collect(cwd, cwd, 0, &mut paths);
    paths.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reads_c_files() {
        let dir = std::env::temp_dir().join("c-man-snap-test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.c"), "int main(void) { return 0; }").unwrap();
        std::fs::write(dir.join("notes.txt"), "pas du C").unwrap();
        let snap = snapshot(&dir);
        assert_eq!(snap.files.len(), 1);
        assert!(snap.files[0].0.ends_with("main.c"));
        assert!(snap.to_prompt().contains("int main"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snapshot_empty_dir() {
        let dir = std::env::temp_dir().join("c-man-snap-empty");
        std::fs::create_dir_all(&dir).unwrap();
        let snap = snapshot(&dir);
        assert!(snap.files.is_empty());
        assert!(snap.to_prompt().contains("aucun fichier"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
