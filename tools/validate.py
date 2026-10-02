#!/usr/bin/env python3
"""Validateur de fiches c-man. Usage: python3 tools/validate.py content/*.toml"""
import sys
import tomllib
import pathlib

REQUIRED = ["id", "title", "category", "difficulty", "synopsis", "description", "example"]
CATEGORIES = {"demarrer", "langage", "memoire", "libc", "outils", "piscine", "bash", "web"}

IDS = {
    # demarrer
    "bienvenue", "roadmap", "lire-un-man", "la-norme", "moulinette",
    # langage
    "variables-types", "operateurs", "conditions", "boucles", "fonctions",
    "portee-variables", "main-argc-argv", "tableaux", "chaines-caracteres",
    "operateurs-bitwise", "structures", "typedef", "enum", "union", "const",
    "recursivite", "pointeurs-de-fonctions", "preprocesseur",
    # memoire
    "pointeurs", "arithmetique-pointeurs", "pointeurs-vs-tableaux",
    "stack-vs-heap", "malloc", "free", "calloc-realloc", "fuites-memoire",
    "segfault", "undefined-behavior", "debordements-entiers",
    # libc
    "write", "read", "open-close", "printf", "scanf", "putchar", "exit",
    "strlen", "strcpy-strncpy", "strcmp-strncmp", "strcat-strncat",
    "strchr-strstr", "strdup", "memcpy-memmove", "memset", "ctype",
    "atoi-strtol", "math", "qsort", "abs",
    # outils
    "gcc", "flags-compilation", "headers-include-guards", "makefile",
    "linking", "gdb", "valgrind",
    # piscine
    "erreurs-frequentes", "methodologie-debug", "gestion-errno", "conseils-piscine",
    # v2
    "fichiers-stdio", "stat-opendir", "bibliotheques", "environnement",
    "fonctions-variadiques", "bonnes-pratiques", "criterion",
    # bash
    "bash-bases", "pipes-redirections", "variables-shell", "scripts-bash",
    "conditions-bash", "boucles-bash", "commandes-texte", "permissions",
    "processus", "git-bases",
    # web
    "html-bases", "html-formulaires", "css-bases", "css-flexbox",
    "css-boxmodel", "js-bases", "js-dom", "js-fonctions", "js-async",
    "js-evenements",
    # vague 2
    "listes-chainees",
    "css-position", "css-responsive", "js-localstorage", "js-formulaires",
    "html-semantique",
    # vague 3
    "web-devtools", "http-bases", "js-arrays",
    # vague 4
    "js-objets", "css-transitions", "js-modules",
}


def check(path: pathlib.Path) -> list[str]:
    errors = []
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        return [f"TOML invalide: {e}"]
    for key in REQUIRED:
        if key not in data:
            errors.append(f"champ manquant: {key}")
    if not errors and data.get("id") != path.stem:
        errors.append(f"id '{data.get('id')}' != nom de fichier '{path.stem}'")
    if data.get("category") not in CATEGORIES:
        errors.append(f"categorie invalide: {data.get('category')!r}")
    d = data.get("difficulty")
    if not isinstance(d, int) or not (1 <= d <= 5):
        errors.append("difficulty doit etre un entier entre 1 et 5")
    if "piscine_day" in data and not isinstance(data["piscine_day"], str):
        errors.append("piscine_day doit etre une chaine (ex: \"J03\")")
    for key in ("tags", "gotchas", "exercises", "related"):
        if key in data and not (
            isinstance(data[key], list) and all(isinstance(x, str) for x in data[key])
        ):
            errors.append(f"{key} doit etre une liste de chaines")
    for r in data.get("related", []):
        if r not in IDS:
            errors.append(f"related inconnu: {r!r}")
    for key in ("description", "example"):
        v = data.get(key, "")
        if isinstance(v, str) and len(v.strip()) < 80:
            errors.append(f"{key} trop court (<80 caracteres)")
    return errors


def main() -> int:
    bad = 0
    for arg in sys.argv[1:]:
        p = pathlib.Path(arg)
        errs = check(p)
        if errs:
            bad = 1
            for e in errs:
                print(f"{p.name}: {e}")
        else:
            print(f"{p.name}: OK")
    if len(sys.argv) < 2:
        print("usage: validate.py content/*.toml")
        return 2
    return bad


if __name__ == "__main__":
    sys.exit(main())
