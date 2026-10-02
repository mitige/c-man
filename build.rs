use std::collections::HashSet;
use std::{env, fs, path::Path};

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct RawEntry {
    id: String,
    title: String,
    category: String,
    difficulty: u8,
    synopsis: String,
    description: String,
    example: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    piscine_day: Option<String>,
    #[serde(default)]
    gotchas: Vec<String>,
    #[serde(default)]
    exercises: Vec<String>,
    #[serde(default)]
    related: Vec<String>,
}

const CATEGORIES: [&str; 8] = ["demarrer", "langage", "memoire", "libc", "outils", "piscine", "bash", "web"];

fn collect_toml(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<_> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect(),
        Err(_) => vec![],
    };
    paths.sort();
    paths
}

fn main() {
    println!("cargo:rerun-if-changed=content");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR manquant");
    let dir = Path::new("content");

    let paths = collect_toml(dir);

    let mut entries: Vec<(String, String)> = vec![]; // (id, contenu toml)
    let mut ids: HashSet<String> = HashSet::new();
    for path in &paths {
        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{}: lecture impossible: {e}", path.display()));
        let parsed: RawEntry = toml::from_str(&text)
            .unwrap_or_else(|e| panic!("{}: TOML invalide: {e}", path.display()));
        let stem = path.file_stem().unwrap().to_str().unwrap();
        assert_eq!(
            parsed.id, stem,
            "{}: le champ id doit etre identique au nom du fichier",
            path.display()
        );
        assert!(
            CATEGORIES.contains(&parsed.category.as_str()),
            "{}: categorie invalide {:?} (attendu: {:?})",
            path.display(),
            parsed.category,
            CATEGORIES
        );
        assert!(
            (1..=5).contains(&parsed.difficulty),
            "{}: difficulty doit etre entre 1 et 5",
            path.display()
        );
        assert!(
            ids.insert(parsed.id.clone()),
            "{}: id duplique {}",
            path.display(),
            parsed.id
        );
        entries.push((parsed.id, text));
    }

    // verification croisee des liens `related`
    for (id, text) in &entries {
        let parsed: RawEntry = toml::from_str(text).unwrap();
        for rel in &parsed.related {
            assert!(
                ids.contains(rel),
                "content/{id}.toml: related inconnu: {rel:?}"
            );
        }
    }

    let mut gen = String::from("pub static CONTENT_FILES: &[&str] = &[\n");
    for (_, text) in &entries {
        gen.push_str(&format!("{text:?},\n"));
    }
    gen.push_str("];\n");

    // banque de quiz : content/quiz/*.toml (validation structurelle)
    let quiz_dir = dir.join("quiz");
    let quiz_paths = collect_toml(&quiz_dir);
    gen.push_str("pub static QUIZ_FILES: &[&str] = &[\n");
    for path in &quiz_paths {
        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{}: lecture impossible: {e}", path.display()));
        validate_quiz(path, &text, &ids);
        gen.push_str(&format!("{text:?},\n"));
    }
    gen.push_str("];\n");
    fs::write(Path::new(&out_dir).join("content.rs"), gen).unwrap();
}

fn validate_quiz(path: &Path, text: &str, fiche_ids: &HashSet<String>) {
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct RawQuiz {
        id: String,
        category: String,
        difficulty: u8,
        question: String,
        choices: Vec<String>,
        answer: usize,
        explanation: String,
        #[serde(default)]
        related: Vec<String>,
    }
    let parsed: RawQuiz = toml::from_str(text)
        .unwrap_or_else(|e| panic!("{}: TOML invalide: {e}", path.display()));
    let stem = path.file_stem().unwrap().to_str().unwrap();
    assert_eq!(parsed.id, stem, "{}: id != nom de fichier", path.display());
    assert!(parsed.id.starts_with("q-"), "{}: id de quiz doit commencer par q-", path.display());
    assert!(
        ["langage", "memoire", "libc", "outils", "piscine", "bash", "web"].contains(&parsed.category.as_str()),
        "{}: categorie de quiz invalide",
        path.display()
    );
    assert!((1..=5).contains(&parsed.difficulty), "{}: difficulty hors 1..5", path.display());
    assert!(parsed.choices.len() >= 2, "{}: il faut au moins 2 choix", path.display());
    assert!(parsed.answer < parsed.choices.len(), "{}: answer hors limites", path.display());
    for rel in &parsed.related {
        assert!(fiche_ids.contains(rel), "{}: related inconnu: {rel:?}", path.display());
    }
}
