# c-man

> Ta piscine Epitech, dans le terminal. Documentation, exercices vérifiés, norme, et un éditeur — tout au clavier, sans quitter ton shell.

c-man est l'environnement complet d'un piscinier qui veut tout finir en premier : des fiches de doc, des exercices dont le code est **vraiment testé**, le checker de Norme Epitech, des quiz, des flashcards, et un éditeur intégré. En français, sobre, rapide.

## Pourquoi

Parce que la piscine, c'est 12h par jour dans un terminal. c-man met tout ce dont tu as besoin à une commande — sans navigateur, sans quitter ton flux.

- **Apprends** : `c-man malloc` — la fiche, l'exemple, les pièges, les exercices.
- **Pratique** : `c-man exo my_strcpy` — ton code est compilé et testé pour de vrai.
- **Vérifie** : `c-man check .` — la Norme Epitech, HTML, CSS, JS en un passage.
- **Révise** : `c-man quiz`, flashcards, examen chronométré.
- **Structure ta journée** : `c-man morning`, `c-man next`, `c-man focus`, `c-man recap`.

## Installer

### Arch (paquet)
```sh
sudo pacman -U c-man-<version>-x86_64.pkg.tar.zst
```

### Binaire précompilé (toute distro Linux x86_64)
Depuis les [Releases](../../releases) :
```sh
curl -LO https://github.com/mitige/c-man/releases/latest/download/c-man
chmod +x c-man && sudo mv c-man /usr/local/bin/
```

### Depuis les sources (partout où Rust est)
```sh
git clone https://github.com/mitige/c-man && cd c-man
cargo install --path . --locked
```

Vérifie ton setup : `c-man doctor`.

## L'essentiel

```sh
c-man                    # la TUI plein écran
c-man <sujet>            # la fiche, comme man
c-man exo                # les exercices guidés (code vérifié)
c-man check .            # norme + web, en un passage
c-man morning            # la routine du matin
c-man edit fichier.c     # l'éditeur epitech-nano
```

TUI : `j`/`k` navigue, `Entrée` ouvre, `Ctrl+O` les outils, `?` l'aide.
Éditeur : `Tab` = 4 espaces, `^S` sauve, `^Q`/`^X` quitte, `^F` cherche, `^G` ligne, `^Z` annule.

## Contenu

- **104 fiches** : C (langage, mémoire, libc), bash, web (HTML/CSS/JS)
- **42 exercices** dont le code est compilé et testé — C, JS, bash, HTML, CSS, DOM
- **67 questions** de quiz · **486 flashcards** (répétition espacée)
- Checker de **Norme Epitech** (auto-correction incluse) + validation **HTML / CSS / JS**
- Suivi de progression : série de jours, points fragiles, exercices faits

## Licence

MIT — fais-en bon usage. Bonne piscine.
