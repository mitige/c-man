//! Configuration utilisateur : ~/.config/c-man/config.toml
//!
//! Toutes les clés sont optionnelles ; les valeurs par défaut suivent la
//! piscine Epitech classique.

use crate::norme::NormeConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Prénom Nom pour l'en-tête Epitech généré par `c-man fix`.
    pub name: Option<String>,
    /// Login Epitech (pour l'en-tête).
    pub login: Option<String>,
    /// Commande pour lancer dsh (ex: "pnpm dsh" ou "dsh").
    pub dsh_command: Option<String>,
    /// Répertoire depuis lequel lancer dsh (ex: le checkout deepseek-harness).
    pub dsh_dir: Option<String>,
    /// Modèle IA (défaut : kimi-k3).
    pub ai_model: Option<String>,
    /// Niveau d'aide IA par défaut (1 socratique .. 5 analyse complète).
    pub ai_help_level: Option<u8>,
    /// Agent pilant la complétion d'epitech-nano.
    pub nano_agent: Option<String>,
    /// Modèle IA de l'éditeur (défaut : deepseek-v4.1-flash, rapide).
    pub nano_model: Option<String>,
    #[serde(flatten)]
    pub norme: NormeSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NormeSection {
    pub max_columns: usize,
    pub max_function_lines: usize,
    pub max_functions_per_file: usize,
    pub forbid_for: bool,
    pub forbid_ternary: bool,
    pub forbid_switch: bool,
    pub forbid_goto: bool,
    pub return_parens: bool,
    pub comments_in_function: bool,
}

impl Default for NormeSection {
    fn default() -> Self {
        let c = NormeConfig::default();
        Self {
            max_columns: c.max_columns,
            max_function_lines: c.max_function_lines,
            max_functions_per_file: c.max_functions_per_file,
            forbid_for: c.forbid_for,
            forbid_ternary: c.forbid_ternary,
            forbid_switch: c.forbid_switch,
            forbid_goto: c.forbid_goto,
            return_parens: c.return_parens,
            comments_in_function: c.comments_in_function,
        }
    }
}

impl NormeSection {
    pub fn to_checker(&self) -> NormeConfig {
        NormeConfig {
            max_columns: self.max_columns,
            max_function_lines: self.max_function_lines,
            max_functions_per_file: self.max_functions_per_file,
            forbid_for: self.forbid_for,
            forbid_ternary: self.forbid_ternary,
            forbid_switch: self.forbid_switch,
            forbid_goto: self.forbid_goto,
            return_parens: self.return_parens,
            comments_in_function: self.comments_in_function,
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs_config().join("c-man").join("config.toml")
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                .join(".local")
                .join("share")
        });
    base.join("c-man")
}

fn dirs_config() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                .join(".config")
        })
}

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
            eprintln!("c-man: config {} invalide: {e}", path.display());
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

/// Persiste l'agent de complétion de nano dans config.toml.
pub fn save_nano_agent(agent: &str) {
    save_key("nano_agent", agent);
}

/// Persiste le modèle IA de l'éditeur.
pub fn save_nano_model(model: &str) {
    save_key("nano_model", model);
}

/// Persiste le modèle IA choisi dans config.toml (sans écraser le reste).
pub fn save_ai_model(model: &str) {
    save_key("ai_model", model);
}

/// Écrit ou remplace une clé dans config.toml (préserve le reste).
fn save_key(key: &str, value: &str) {
    let path = config_path();
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let new_line = format!("{key} = \"{value}\"");
    let mut done = false;
    for line in &mut lines {
        let t = line.trim_start();
        if t.starts_with(key) || t.starts_with(&format!("# {key}")) {
            *line = new_line.clone();
            done = true;
            break;
        }
    }
    if !done {
        lines.push(String::new());
        lines.push(new_line);
    }
    let _ = std::fs::write(&path, lines.join("\n") + "\n");
}

/// Crée le fichier de config avec les valeurs commentées si absent.
pub fn ensure_exists() -> std::io::Result<PathBuf> {
    let path = config_path();
    if path.exists() {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, DEFAULT_CONFIG)?;
    Ok(path)
}

pub const DEFAULT_CONFIG: &str = r##"# Configuration c-man

# Identité pour l'en-tête Epitech généré par `c-man fix --header` :
# name = "Prénom Nom"
# login = "prenom.nom"          # login Epitech (pour <login@epitech.eu>)

# Intégration DSH (détectée automatiquement si dsh est dans le PATH) :
# dsh_command = "pnpm dsh"       # ou "dsh" si installé globalement
# dsh_dir = "~/deepseek-harness" # répertoire de travail pour lancer dsh

# IA (Kimi K3 via le provider du projet) :
# ai_model = "kimi-k3"
# ai_help_level = 2              # 1 socratique .. 5 analyse complète

# Règles de la Norme (valeurs = défauts piscine) :
# max_columns = 80
# max_function_lines = 25
# max_functions_per_file = 5
# forbid_for = true
# forbid_ternary = true
# forbid_switch = false
# forbid_goto = true
# return_parens = false
# comments_in_function = false
"##;
