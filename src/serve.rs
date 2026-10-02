//! Serveur statique minimal (std uniquement) — pour prévisualiser le web.
//!
//! `c-man serve` : sert le dossier courant sur http://127.0.0.1:8080 avec le
//! bon Content-Type, pour ouvrir ton HTML/CSS/JS dans le navigateur.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Résout la requête GET vers un fichier (sécurisé : pas de sortie du dossier).
fn resolve(root: &Path, url_path: &str) -> Option<PathBuf> {
    let clean = url_path.trim_start_matches('/');
    let clean = if clean.is_empty() { "index.html" } else { clean };
    let candidate = root.join(clean);
    // sécurité : refuse ../ qui sortirait du dossier servi
    let canon = candidate.canonicalize().ok()?;
    let root_canon = root.canonicalize().ok()?;
    if !canon.starts_with(&root_canon) {
        return None;
    }
    if canon.is_dir() {
        let idx = canon.join("index.html");
        return if idx.exists() { Some(idx) } else { None };
    }
    Some(canon)
}

/// Sert le dossier jusqu'à Ctrl+C. Retourne le port réellement utilisé.
pub fn serve(dir: &Path, port: u16) -> std::io::Result<u16> {
    let mut listener = None;
    for p in port..port + 10 {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", p)) {
            listener = Some(l);
            if p != port {
                println!("  (port {port} pris, j'écoute sur {p})");
            }
            break;
        }
    }
    let Some(listener) = listener else {
        eprintln!("c-man serve: aucun port libre entre {port} et {}", port + 9);
        return Ok(0);
    };
    let actual = listener.local_addr()?.port();
    println!("  🌐 http://127.0.0.1:{actual}");
    println!("  sert {} — Ctrl+C pour arrêter", dir.display());
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut buf = [0u8; 4096];
        let n = match stream.read(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let req = String::from_utf8_lossy(&buf[..n]);
        // première ligne : GET /chemin HTTP/1.1
        let path = req
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("/");
        let path = path.split(['?', '#']).next().unwrap_or("/");
        match resolve(dir, path) {
            Some(file) if file.is_file() => {
                let body = std::fs::read(&file).unwrap_or_default();
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    content_type(&file),
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
            _ => {
                let body = "404 — pas trouvé\n";
                let head = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body.as_bytes());
            }
        }
    }
    Ok(actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_ok() {
        assert_eq!(content_type(Path::new("a.html")), "text/html; charset=utf-8");
        assert_eq!(content_type(Path::new("a.css")), "text/css; charset=utf-8");
        assert_eq!(content_type(Path::new("a.js")), "application/javascript; charset=utf-8");
        assert_eq!(content_type(Path::new("a.bin")), "application/octet-stream");
    }

    #[test]
    fn resolve_bloque_traversal() {
        let root = std::env::temp_dir();
        assert!(resolve(&root, "/../../etc/passwd").is_none());
    }
}
