# c-man — Design

## Objectif
Un `man` moderne dédié au langage C, pensé pour un étudiant en piscine Epitech (8h45→21h, 1 mois). Apprentissage rapide ET profond : chaque fiche va à l'essentiel, montre un exemple compilable, liste les pièges concrets et propose des micro-exercices façon piscine.

## Décisions
- **Langage** : Rust (demande explicite). Binaire unique `c-man`, contenu embarqué à la compilation (zéro dépendance runtime, fonctionne en TTY pur).
- **TUI** : ratatui + crossterm. Coloration syntaxique : syntect. Recherche floue : fuzzy-matcher.
- **Contenu** : ~60 fiches TOML dans `content/`, validées et embarquées par `build.rs`. Rédaction en français, code en C99 compatible Norme.
- **Packaging Arch** : PKGBUILD → `makepkg` → `.pkg.tar.zst` + installation directe dans `~/.local/bin` (déjà dans le PATH, pas de root requis). `sudo pacman -U` reste possible.

## Modes CLI
- `c-man` → navigateur TUI (recherche fuzzy `/`, navigation `j/k`, `?` aide)
- `c-man <sujet>` → rendu direct dans le terminal (ANSI si TTY, texte brut si pipé) — aussi rapide que `man`
- `c-man -t <sujet>` → ouvre la TUI directement sur le sujet
- `c-man -l [cat]` → liste les fiches
- `c-man -s <query>` → recherche plein texte, résultats en liste
- `c-man --random` → fiche aléatoire (rituel quotidien)
- `c-man roadmap` → parcours piscine jour par jour

## Format de fiche (TOML)
```toml
id = "printf"
title = "printf — affichage formaté"
category = "libc"            # demarrer|langage|memoire|libc|outils|piscine
tags = ["io", "format"]
difficulty = 2               # 1..5
piscine_day = "J03"          # optionnel
synopsis = "int printf(const char *format, ...);"
description = """markdown : ## sections, **gras**, `code`, ```c blocs, - listes, > notes"""
example = """/* C compilable, commenté en français */"""
gotchas = ["piège concret"]
exercises = ["micro-exercice façon piscine"]
related = ["fprintf", "scanf"]
```

## Catégories (~60 fiches)
- **demarrer** (5) : bienvenue, roadmap, lire-un-man, la-norme, moulinette
- **langage** (18) : types, opérateurs, conditions, boucles, fonctions, portée, argc/argv, tableaux, chaînes, bitwise, struct, typedef, enum, union, const, récursivité, pointeurs de fonctions, préprocesseur
- **memoire** (11) : pointeurs, arithmétique, pointeurs vs tableaux, stack/heap, malloc, free, calloc/realloc, fuites, segfault, UB, débordements
- **libc** (20) : write, read, open/close, printf, scanf, putchar, exit, string.h (8 fiches groupées), ctype, atoi/strtol, math, qsort, abs
- **outils** (7) : gcc, flags, headers/include guards, Makefile, linking, gdb, valgrind
- **piscine** (4) : erreurs fréquentes, méthodologie debug, errno, conseils

## TUI
```
┌ c-man ─ [recherche fuzzy____________] ─────────────┐
│ 📘 Langage C        │ printf — affichage formaté    │
│   ▸ variables-types │ ┌─ SYNOPSIS ────────────────┐ │
│   ▸ ...             │ │ int printf(...)           │ │
│ 🧠 Mémoire          │ └───────────────────────────┘ │
│   ▸ pointeurs       │ ## Explication ...            │
│                     │ ┌─ EXEMPLE ─────────────────┐ │
│                     │ │ code coloré syntect       │ │
│                     │ └───────────────────────────┘ │
│                     │ ⚠ Pièges ... 🏋 Exercices ... │
├ j/k naviguer · / chercher · Enter ouvrir · q quitter ┤
```
Thème sombre "piscine night" : accents cyan, pièges en doré, exemples encadrés vert, respect de `NO_COLOR`.

## Erreurs & tests
- Fiche inconnue → suggestions fuzzy ("Vouliez-vous dire … ?")
- build.rs échoue bruyamment si un TOML est invalide (nom de fichier + erreur)
- Tests unitaires : parsing fiches, renderer markdown, recherche
- Smoke test TUI via `script` (pty) : rendu sans panic

## Hors scope (YAGNI)
Complétions shell, man page de c-man, thèmes multiples, réseau/télémétrie.
