//! epitech-nano — l'éditeur CLI de la piscine.
//!
//! Tab = 4 espaces. Complétion IA en texte fantôme. Zéro bruit.

use std::path::PathBuf;

fn main() {
    let file = std::env::args().nth(1).map(PathBuf::from);
    if matches!(
        std::env::args().nth(1).as_deref(),
        Some("-h") | Some("--help") | Some("-V") | Some("--version")
    ) {
        println!(
            "epitech-nano — l'éditeur CLI de la piscine\n\n\
             Usage: epitech-nano [fichier]\n\n\
             Tab        4 espaces (la Norme)\n\
             Ctrl+S     sauvegarder\n\
             Ctrl+Q     quitter\n\
             Ctrl+K/U   couper / coller une ligne\n\
             Ctrl+Espace  complétion IA (texte fantôme)\n\
             Alt+→      accepter la suggestion IA\n\
             Échap      refuser la suggestion"
        );
        return;
    }
    if let Err(e) = c_man::editor::run(file) {
        eprintln!("epitech-nano: {e}");
        std::process::exit(1);
    }
}
