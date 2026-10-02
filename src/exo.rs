//! Exercices guidés avec vérification réelle (compile + tests).

/// Un exercice : la tâche, le squelette à compléter, le harnais de test.
pub struct Exo {
    pub id: &'static str,
    pub title: &'static str,
    pub difficulty: u8,
    pub task: &'static str,
    /// Le prototype imposé.
    pub signature: &'static str,
    /// Le code de départ (à écrire dans le fichier de l'exo).
    pub starter: &'static str,
    /// Le harnais de test (main + assertions) — compilé avec le fichier de l'étudiant.
    pub harness: &'static str,
    /// Indices progressifs (du plus léger au plus direct).
    pub hints: &'static [&'static str],
    /// Fiche liée pour la théorie.
    pub fiche: &'static str,
    /// Langage de l'exercice : "c" (gcc) ou "js" (node).
    pub lang: &'static str,
    /// Nom du fichier de solution (défaut : <id>.c).
    pub solution_file: &'static str,
}

pub const EXOS: &[Exo] = &[
    Exo {
        id: "my_strlen",
        title: "compter les caractères d'une chaîne",
        difficulty: 1,
        task: "Écris `int my_strlen(char const *str)` qui renvoie le nombre de caractères de `str` (sans compter le '\\0').",
        signature: "int my_strlen(char const *str);",
        starter: "int my_strlen(char const *str)\n{\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_strlen(char const *str);\n\nstatic int test(char *s, int attendu)\n{\n    int got = my_strlen(s);\n    if (got != attendu) {\n        printf(\"✗ my_strlen(\\\"%s\\\") = %d, attendu %d\\n\", s, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_strlen(\\\"%s\\\") = %d\\n\", s, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", 0);\n    ko += test(\"a\", 1);\n    ko += test(\"abc\", 3);\n    ko += test(\"piscine epitech\", 15);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Une chaîne se termine par '\\0'. Parcours-la avec un index.",
            "while (str[i] != '\\0') i++; puis renvoie i.",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strcpy",
        title: "copier une chaîne dans une autre",
        difficulty: 2,
        task: "Écris `char *my_strcpy(char *dest, char const *src)` qui copie `src` dans `dest` (le '\\0` compris) et renvoie `dest`.",
        signature: "char *my_strcpy(char *dest, char const *src);",
        starter: "char *my_strcpy(char *dest, char const *src)\n{\n    // à toi\n    return (dest);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strcpy(char *dest, char const *src);\n\nstatic int test(char *src)\n{\n    char a[64];\n    char b[64];\n    strcpy(a, src);\n    my_strcpy(b, src);\n    if (strcmp(a, b) != 0) {\n        printf(\"✗ my_strcpy sur \\\"%s\\\" → \\\"%s\\\", attendu \\\"%s\\\"\\n\", src, b, a);\n        return (1);\n    }\n    printf(\"✓ my_strcpy sur \\\"%s\\\"\\n\", src);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\");\n    ko += test(\"x\");\n    ko += test(\"hello world\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Copie caractère par caractère jusqu'au '\\0' INCLUS.",
            "La boucle copie, puis n'oublie pas le '\\0' final, puis return dest.",
        ],
        fiche: "strcpy-strncpy",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strcmp",
        title: "comparer deux chaînes",
        difficulty: 2,
        task: "Écris `int my_strcmp(char const *s1, char const *s2)` : 0 si égales, négatif si s1 < s2, positif si s1 > s2.",
        signature: "int my_strcmp(char const *s1, char const *s2);",
        starter: "int my_strcmp(char const *s1, char const *s2)\n{\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nint my_strcmp(char const *s1, char const *s2);\n\nstatic int sign(int x)\n{\n    return ((x > 0) - (x < 0));\n}\n\nstatic int test(char *a, char *b)\n{\n    if (sign(my_strcmp(a, b)) != sign(strcmp(a, b))) {\n        printf(\"✗ my_strcmp(\\\"%s\\\", \\\"%s\\\") = %d\\n\", a, b, my_strcmp(a, b));\n        return (1);\n    }\n    printf(\"✓ my_strcmp(\\\"%s\\\", \\\"%s\\\")\\n\", a, b);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"abc\", \"abc\");\n    ko += test(\"abc\", \"abd\");\n    ko += test(\"abd\", \"abc\");\n    ko += test(\"\", \"\");\n    ko += test(\"a\", \"\");\n    ko += test(\"hello\", \"hello world\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Compare caractère par caractère ; arrête dès qu'ils diffèrent ou à un '\\0'.",
            "while (s1[i] == s2[i] && s1[i] != '\\0') i++; return (s1[i] - s2[i]);",
        ],
        fiche: "strcmp-strncmp",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_putchar",
        title: "afficher un caractère avec write",
        difficulty: 1,
        task: "Écris `void my_putchar(char c)` qui affiche le caractère `c` sur la sortie standard en utilisant write.",
        signature: "void my_putchar(char c);",
        starter: "#include <unistd.h>\n\nvoid my_putchar(char c)\n{\n    (void)c;\n    // à toi\n}\n",
        harness: "#define _POSIX_C_SOURCE 200809L\n#include <stdio.h>\n#include <string.h>\n#include <unistd.h>\nvoid my_putchar(char c);\n\nstatic int test(char c)\n{\n    int fds[2];\n    int saved;\n    char buf[8];\n    int n;\n    int ok;\n\n    memset(buf, 0, 8);\n    saved = dup(1);\n    pipe(fds);\n    fflush(stdout);\n    dup2(fds[1], 1);\n    my_putchar(c);\n    fflush(stdout);\n    dup2(saved, 1);\n    close(saved);\n    close(fds[1]);\n    n = read(fds[0], buf, 7);\n    close(fds[0]);\n    ok = (n == 1 && buf[0] == c);\n    if (!ok) {\n        printf(\"✗ my_putchar('%c') → %d octet(s) écrit(s)\\n\", c, n);\n        return (1);\n    }\n    printf(\"✓ my_putchar('%c')\\n\", c);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n\n    ko += test('A');\n    ko += test('z');\n    ko += test('0');\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "write prend 3 arguments : un descripteur de fichier, une adresse, un nombre d'octets.",
            "La sortie standard, c'est le descripteur 1. Le caractère à afficher est dans c : donne son adresse.",
            "write(1, &c, 1);",
        ],
        fiche: "write",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_compute_factorial_rec",
        title: "factorielle récursive",
        difficulty: 2,
        task: "Écris `int my_compute_factorial_rec(int nb)` qui renvoie nb! de façon RÉCURSIVE. Renvoie 0 si `nb` est négatif ou nul.",
        signature: "int my_compute_factorial_rec(int nb);",
        starter: "int my_compute_factorial_rec(int nb)\n{\n    (void)nb;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_compute_factorial_rec(int nb);\n\nstatic int test(int nb, int attendu)\n{\n    int got = my_compute_factorial_rec(nb);\n    if (got != attendu) {\n        printf(\"✗ my_compute_factorial_rec(%d) = %d, attendu %d\\n\", nb, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_compute_factorial_rec(%d) = %d\\n\", nb, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(0, 0);\n    ko += test(-5, 0);\n    ko += test(1, 1);\n    ko += test(2, 2);\n    ko += test(5, 120);\n    ko += test(10, 3628800);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "factorielle(n) = n × factorielle(n - 1). Quel est le cas d'arrêt ?",
            "Commence par les gardes : if (nb <= 0) return (0); puis if (nb == 1) return (1);",
            "return (nb * my_compute_factorial_rec(nb - 1));",
        ],
        fiche: "recursivite",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strcat",
        title: "concaténer deux chaînes",
        difficulty: 2,
        task: "Écris `char *my_strcat(char *dest, char const *src)` qui ajoute `src` à la fin de `dest` et renvoie `dest`.",
        signature: "char *my_strcat(char *dest, char const *src);",
        starter: "char *my_strcat(char *dest, char const *src)\n{\n    (void)src;\n    // à toi\n    return (dest);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strcat(char *dest, char const *src);\n\nstatic int test(char *a, char *b)\n{\n    char x[128];\n    char y[128];\n    char *ret;\n\n    strcpy(x, a);\n    strcpy(y, a);\n    strcat(x, b);\n    ret = my_strcat(y, b);\n    if (strcmp(x, y) != 0 || ret != y) {\n        printf(\"✗ my_strcat(\\\"%s\\\", \\\"%s\\\") → \\\"%s\\\", attendu \\\"%s\\\"\\n\", a, b, y, x);\n        return (1);\n    }\n    printf(\"✓ my_strcat(\\\"%s\\\", \\\"%s\\\") → \\\"%s\\\"\\n\", a, b, y);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", \"\");\n    ko += test(\"hello\", \"\");\n    ko += test(\"\", \"world\");\n    ko += test(\"foo\", \"bar\");\n    ko += test(\"epi\", \"tech\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Va d'abord au bout de dest (jusqu'au '\\0'), puis recopie src à partir de là.",
            "C'est my_strlen + my_strcpy : strcpy(dest + strlen(dest), src), mais à la main.",
            "Deux index : i avance jusqu'au '\\0' de dest, puis j copie src[j] dans dest[i+j], '\\0' compris.",
        ],
        fiche: "strcat-strncat",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_atoi",
        title: "convertir une chaîne en int",
        difficulty: 3,
        task: "Écris `int my_atoi(char const *str)` qui convertit `str` en int : ignore les espaces au début, gère les signes '+' et '-' (même plusieurs), puis lit les chiffres et s'arrête au premier non-chiffre.",
        signature: "int my_atoi(char const *str);",
        starter: "int my_atoi(char const *str)\n{\n    (void)str;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_atoi(char const *str);\n\nstatic int test(char *s, int attendu)\n{\n    int got = my_atoi(s);\n    if (got != attendu) {\n        printf(\"✗ my_atoi(\\\"%s\\\") = %d, attendu %d\\n\", s, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_atoi(\\\"%s\\\") = %d\\n\", s, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"42\", 42);\n    ko += test(\"-42\", -42);\n    ko += test(\"   123\", 123);\n    ko += test(\"+7\", 7);\n    ko += test(\"--42\", 42);\n    ko += test(\"-+12\", -12);\n    ko += test(\"  3abc\", 3);\n    ko += test(\"abc\", 0);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Trois phases : sauter les espaces, lire les signes, lire les chiffres.",
            "Chaque '-' inverse le signe. Un caractère chiffre c donne le nombre c - '0'.",
            "n = n * 10 + (str[i] - '0'); tant que str[i] est entre '0' et '9'.",
        ],
        fiche: "atoi-strtol",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_swap",
        title: "échanger deux entiers via pointeurs",
        difficulty: 1,
        task: "Écris `void my_swap(int *a, int *b)` qui échange les valeurs pointées par `a` et `b`.",
        signature: "void my_swap(int *a, int *b);",
        starter: "void my_swap(int *a, int *b)\n{\n    (void)a;\n    (void)b;\n    // à toi\n}\n",
        harness: "#include <stdio.h>\nvoid my_swap(int *a, int *b);\n\nstatic int test(int x, int y)\n{\n    int a = x;\n    int b = y;\n\n    my_swap(&a, &b);\n    if (a != y || b != x) {\n        printf(\"✗ my_swap(%d, %d) → a=%d b=%d\\n\", x, y, a, b);\n        return (1);\n    }\n    printf(\"✓ my_swap(%d, %d) → a=%d b=%d\\n\", x, y, a, b);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(1, 2);\n    ko += test(-5, 42);\n    ko += test(7, 7);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "*a désigne la valeur pointée par a. Tu peux la lire et l'écrire.",
            "Il faut une variable temporaire pour ne pas écraser une valeur.",
            "int tmp = *a; *a = *b; *b = tmp;",
        ],
        fiche: "pointeurs",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_revstr",
        title: "inverser une chaîne en place",
        difficulty: 2,
        task: "Écris `char *my_revstr(char *str)` qui inverse `str` sur place (\"abc\" devient \"cba\") et renvoie `str`.",
        signature: "char *my_revstr(char *str);",
        starter: "char *my_revstr(char *str)\n{\n    // à toi\n    return (str);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_revstr(char *str);\n\nstatic int test(char *src, char *attendu)\n{\n    char buf[64];\n    char *ret;\n\n    strcpy(buf, src);\n    ret = my_revstr(buf);\n    if (strcmp(buf, attendu) != 0 || ret != buf) {\n        printf(\"✗ my_revstr(\\\"%s\\\") → \\\"%s\\\", attendu \\\"%s\\\"\\n\", src, buf, attendu);\n        return (1);\n    }\n    printf(\"✓ my_revstr(\\\"%s\\\") → \\\"%s\\\"\\n\", src, buf);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", \"\");\n    ko += test(\"a\", \"a\");\n    ko += test(\"ab\", \"ba\");\n    ko += test(\"hello\", \"olleh\");\n    ko += test(\"epitech\", \"hcetipe\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Deux index : un au début, un à la fin. Échange les caractères et rapproche-les.",
            "La fin, c'est my_strlen(str) - 1. Attention à ne pas toucher le '\\0'.",
            "while (i < j) { échange str[i] et str[j] ; i++; j--; } — my_swap peut servir !",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_list_size",
        title: "compter les nœuds d'une liste chaînée",
        difficulty: 3,
        task: "Écris `int my_list_size(t_list *list)` qui renvoie le nombre de nœuds de la liste chaînée `list` (0 si la liste est vide).",
        signature: "int my_list_size(t_list *list);",
        starter: "typedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nint my_list_size(t_list *list)\n{\n    (void)list;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\n#include <stddef.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nint my_list_size(t_list *list);\n\nstatic int test(int n, int attendu)\n{\n    t_list nodes[5];\n    t_list *head;\n    int i;\n    int got;\n\n    i = 0;\n    head = NULL;\n    while (i < n) {\n        nodes[i].data = NULL;\n        nodes[i].next = head;\n        head = &nodes[i];\n        i++;\n    }\n    got = my_list_size(head);\n    if (got != attendu) {\n        printf(\"✗ my_list_size(%d nœud(s)) = %d, attendu %d\\n\", n, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_list_size(%d nœud(s)) = %d\\n\", n, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(0, 0);\n    ko += test(1, 1);\n    ko += test(3, 3);\n    ko += test(5, 5);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Une liste chaînée se parcourt en suivant le champ next jusqu'à NULL.",
            "int n = 0; while (list != NULL) { n++; list = list->next; }",
            "N'oublie pas de renvoyer n après la boucle.",
        ],
        fiche: "listes-chainees",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "js_fizzbuzz",
        title: "fizzbuzz en JS",
        difficulty: 1,
        task: "Écris une fonction `fizzbuzz(n)` qui renvoie un tableau : pour 1..n, « fizz » si multiple de 3, « buzz » si multiple de 5, « fizzbuzz » si les deux, sinon le nombre.",
        signature: "function fizzbuzz(n)",
        starter: "function fizzbuzz(n) {
    // à toi
}
",
        harness: "const out = fizzbuzz(5);
const ok = JSON.stringify(out) === JSON.stringify([1,2,'fizz',4,'buzz']);
if (ok) { console.log('✓ fizzbuzz(5)'); } else { console.log('✗ fizzbuzz(5) =', out); process.exit(84); }
const out2 = fizzbuzz(15);
const ok2 = out2[14] === 'fizzbuzz' && out2[2] === 'fizz' && out2[4] === 'buzz';
if (ok2) { console.log('✓ fizzbuzz(15)'); } else { console.log('✗ fizzbuzz(15)'); process.exit(84); }
",
        hints: &[
            "Boucle de 1 à n, et teste la divisibilité (modulo %).",
            "Teste 15 (les deux) AVANT 3 et 5 séparément.",
        ],
        fiche: "js-bases",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "js_map_filter",
        title: "transformer et filtrer un tableau",
        difficulty: 2,
        task: "Écris `adultes(personnes)` qui prend un tableau d'objets { nom, age } et renvoie les noms (en majuscules) des majeurs (age >= 18).",
        signature: "function adultes(personnes)",
        starter: "function adultes(personnes) {
    // à toi
}
",
        harness: "const p = [{nom:'a',age:17},{nom:'bob',age:20},{nom:'cy',age:15},{nom:'dan',age:30}];
const out = adultes(p);
const ok = JSON.stringify(out) === JSON.stringify(['BOB','DAN']);
if (ok) { console.log('✓ adultes filtre et transforme'); } else { console.log('✗ adultes =', out); process.exit(84); }
const ok2 = JSON.stringify(adultes([])) === '[]';
if (ok2) { console.log('✓ tableau vide'); } else { process.exit(84); }
",
        hints: &[
            "filter() pour garder les majeurs, puis map() pour les noms.",
            "personnes.filter(p => p.age >= 18).map(p => p.nom.toUpperCase())",
        ],
        fiche: "js-arrays",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "my_strncmp",
        title: "comparer deux chaînes sur n caractères",
        difficulty: 2,
        task: "Écris `int my_strncmp(char const *s1, char const *s2, int n)` qui compare au plus `n` caractères de `s1` et `s2` : 0 s'ils sont égaux, négatif si s1 < s2, positif si s1 > s2.",
        signature: "int my_strncmp(char const *s1, char const *s2, int n);",
        starter: "int my_strncmp(char const *s1, char const *s2, int n)\n{\n    (void)s1;\n    (void)s2;\n    (void)n;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nint my_strncmp(char const *s1, char const *s2, int n);\n\nstatic int sign(int x)\n{\n    return ((x > 0) - (x < 0));\n}\n\nstatic int test(char *a, char *b, int n)\n{\n    if (sign(my_strncmp(a, b, n)) != sign(strncmp(a, b, (size_t)n))) {\n        printf(\"✗ my_strncmp(\\\"%s\\\", \\\"%s\\\", %d) = %d\\n\", a, b, n, my_strncmp(a, b, n));\n        return (1);\n    }\n    printf(\"✓ my_strncmp(\\\"%s\\\", \\\"%s\\\", %d)\\n\", a, b, n);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"abc\", \"abc\", 3);\n    ko += test(\"abc\", \"abd\", 2);\n    ko += test(\"abc\", \"abd\", 3);\n    ko += test(\"abd\", \"abc\", 3);\n    ko += test(\"abc\", \"xyz\", 0);\n    ko += test(\"abc\", \"abcdef\", 6);\n    ko += test(\"hello\", \"hell\", 4);\n    ko += test(\"hell\", \"hello\", 5);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Comme my_strcmp, mais avec un compteur : on s'arrête après n comparaisons.",
            "while (i < n && s1[i] == s2[i] && s1[i] != '\\0') i++; puis gère le cas où i atteint n.",
            "Si i == n (ou n <= 0), renvoie 0 ; sinon return ((unsigned char)s1[i] - (unsigned char)s2[i]);",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_put_nbr",
        title: "afficher un nombre avec write",
        difficulty: 3,
        task: "Écris `void my_put_nbr(int nb)` qui affiche `nb` en décimal sur la sortie standard (avec write, rien d'autre). Gère le 0, les négatifs et INT_MIN (-2147483648).",
        signature: "void my_put_nbr(int nb);",
        starter: "#include <unistd.h>\n\nvoid my_put_nbr(int nb)\n{\n    (void)nb;\n    // à toi\n}\n",
        harness: "#define _POSIX_C_SOURCE 200809L\n#include <limits.h>\n#include <stdio.h>\n#include <string.h>\n#include <unistd.h>\nvoid my_put_nbr(int nb);\n\nstatic int test(int nb, char const *attendu)\n{\n    int fds[2];\n    int saved;\n    char buf[32];\n    int n;\n    int ok;\n\n    memset(buf, 0, 32);\n    saved = dup(1);\n    pipe(fds);\n    fflush(stdout);\n    dup2(fds[1], 1);\n    my_put_nbr(nb);\n    fflush(stdout);\n    dup2(saved, 1);\n    close(saved);\n    close(fds[1]);\n    n = read(fds[0], buf, 31);\n    close(fds[0]);\n    ok = (n == (int)strlen(attendu) && strcmp(buf, attendu) == 0);\n    if (!ok) {\n        printf(\"✗ my_put_nbr(%d) → \\\"%s\\\", attendu \\\"%s\\\"\\n\", nb, buf, attendu);\n        return (1);\n    }\n    printf(\"✓ my_put_nbr(%d) → \\\"%s\\\"\\n\", nb, buf);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n\n    ko += test(0, \"0\");\n    ko += test(42, \"42\");\n    ko += test(-42, \"-42\");\n    ko += test(1000, \"1000\");\n    ko += test(INT_MIN, \"-2147483648\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Écris d'abord un petit my_putchar (write(1, &c, 1)), puis affiche les chiffres du plus significatif au moins significatif.",
            "Récursion : si nb >= 10, affiche d'abord nb / 10 ; puis affiche le chiffre '0' + nb % 10. Si nb < 0, affiche '-' d'abord.",
            "Piège : -INT_MIN déborde d'un int. Convertis nb en long (ou unsigned) AVANT de changer son signe.",
        ],
        fiche: "write",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strstr",
        title: "chercher une sous-chaîne",
        difficulty: 3,
        task: "Écris `char *my_strstr(char const *str, char const *to_find)` qui renvoie un pointeur sur la première occurrence de `to_find` dans `str`, ou NULL si elle est absente. Si `to_find` est vide, renvoie `str`.",
        signature: "char *my_strstr(char const *str, char const *to_find);",
        starter: "#include <stddef.h>\n\nchar *my_strstr(char const *str, char const *to_find)\n{\n    (void)str;\n    (void)to_find;\n    // à toi\n    return (NULL);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strstr(char const *str, char const *to_find);\n\nstatic int test(char *str, char *to_find)\n{\n    char *attendu = strstr(str, to_find);\n    char *got = my_strstr(str, to_find);\n\n    if (got != attendu) {\n        printf(\"✗ my_strstr(\\\"%s\\\", \\\"%s\\\") : mauvais pointeur renvoyé\\n\", str, to_find);\n        return (1);\n    }\n    printf(\"✓ my_strstr(\\\"%s\\\", \\\"%s\\\")\\n\", str, to_find);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"hello world\", \"world\");\n    ko += test(\"hello world\", \"bye\");\n    ko += test(\"hello world\", \"hello\");\n    ko += test(\"hello world\", \"\");\n    ko += test(\"aaaa\", \"aa\");\n    ko += test(\"\", \"x\");\n    ko += test(\"\", \"\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Cas particulier d'abord : si to_find[0] == '\\0', renvoie (char *)str.",
            "Pour chaque position i de str, vérifie si to_find s'y loge : compare to_find[j] avec str[i + j].",
            "Si to_find[j] == '\\0' en sortie de boucle interne, c'est trouvé : return ((char *)str + i) ; sinon, en fin de str, renvoie NULL.",
        ],
        fiche: "strchr-strstr",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_compute_power_rec",
        title: "puissance récursive",
        difficulty: 2,
        task: "Écris `int my_compute_power_rec(int nb, int p)` qui renvoie `nb` élevé à la puissance `p` de façon RÉCURSIVE. Si `p` est négatif, renvoie 0.",
        signature: "int my_compute_power_rec(int nb, int p);",
        starter: "int my_compute_power_rec(int nb, int p)\n{\n    (void)nb;\n    (void)p;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_compute_power_rec(int nb, int p);\n\nstatic int test(int nb, int p, int attendu)\n{\n    int got = my_compute_power_rec(nb, p);\n    if (got != attendu) {\n        printf(\"✗ my_compute_power_rec(%d, %d) = %d, attendu %d\\n\", nb, p, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_compute_power_rec(%d, %d) = %d\\n\", nb, p, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(2, 0, 1);\n    ko += test(2, 3, 8);\n    ko += test(5, -1, 0);\n    ko += test(0, 0, 1);\n    ko += test(3, 4, 81);\n    ko += test(-2, 3, -8);\n    ko += test(-2, 2, 4);\n    ko += test(10, 5, 100000);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "power(nb, p) = nb × power(nb, p - 1). Quel est le cas d'arrêt ?",
            "Gardes : if (p < 0) return (0); puis if (p == 0) return (1);",
            "return (nb * my_compute_power_rec(nb, p - 1));",
        ],
        fiche: "recursivite",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_list_push",
        title: "ajouter un nœud en tête de liste",
        difficulty: 4,
        task: "Écris `void my_list_push(t_list **list, void *data)` qui crée un nouveau nœud (avec malloc) contenant `data` et l'ajoute EN TÊTE de la liste : après l'appel, `*list` pointe sur ce nouveau nœud.",
        signature: "void my_list_push(t_list **list, void *data);",
        starter: "#include <stdlib.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nvoid my_list_push(t_list **list, void *data)\n{\n    (void)list;\n    (void)data;\n    // à toi\n}\n",
        harness: "#include <stdio.h>\n#include <stdlib.h>\n#include <stddef.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nvoid my_list_push(t_list **list, void *data);\n\nint main(void)\n{\n    t_list *head = NULL;\n    t_list *tmp = NULL;\n    int a = 1;\n    int b = 2;\n    int ko = 0;\n\n    my_list_push(&head, &a);\n    if (head == NULL || head->data != &a || head->next != NULL) {\n        printf(\"✗ après 1 push : la tête est incorrecte\\n\");\n        ko = 1;\n    } else {\n        printf(\"✓ 1er push : la tête pointe sur le nouveau nœud\\n\");\n    }\n    my_list_push(&head, &b);\n    if (head == NULL || head->data != &b || head->next == NULL || head->next->data != &a || head->next->next != NULL) {\n        printf(\"✗ après 2 push : la liste est incorrecte\\n\");\n        ko = 1;\n    } else {\n        printf(\"✓ 2e push : le nouveau nœud est devenu la tête\\n\");\n    }\n    while (head != NULL) {\n        tmp = head;\n        head = head->next;\n        free(tmp);\n    }\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "list est un t_list ** : *list désigne la tête actuelle de la liste (NULL si elle est vide).",
            "Crée le nœud : t_list *n = malloc(sizeof(t_list)); puis remplis n->data et n->next.",
            "L'ordre compte : n->next = *list; PUIS *list = n; (sinon tu perds l'ancienne tête).",
        ],
        fiche: "listes-chainees",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "js_compter",
        title: "compter les voyelles",
        difficulty: 1,
        task: "Écris une fonction `compterVoyelles(chaine)` qui renvoie le nombre de voyelles (a, e, i, o, u — minuscules ou majuscules) contenues dans `chaine`.",
        signature: "function compterVoyelles(chaine)",
        starter: "function compterVoyelles(chaine) {\n    // à toi\n}\n",
        harness: "function check(label, got, attendu) {\n    if (got === attendu) {\n        console.log('✓ ' + label);\n    } else {\n        console.log('✗ ' + label + ' = ' + got + ', attendu ' + attendu);\n        process.exit(84);\n    }\n}\n\ncheck('compterVoyelles(\"\")', compterVoyelles(''), 0);\ncheck('compterVoyelles(\"bonjour\")', compterVoyelles('bonjour'), 3);\ncheck('compterVoyelles(\"xyz\")', compterVoyelles('xyz'), 0);\ncheck('compterVoyelles(\"AEIOU\")', compterVoyelles('AEIOU'), 5);\ncheck('compterVoyelles(\"Piscine Epitech\")', compterVoyelles('Piscine Epitech'), 6);\n",
        hints: &[
            "Parcours les caractères (for (const c of chaine)) et compte ceux qui sont des voyelles.",
            "La chaîne 'aeiouAEIOU' avec .includes(c) couvre minuscules et majuscules.",
            "let n = 0; for (const c of chaine) { if ('aeiouAEIOU'.includes(c)) n++; } return n;",
        ],
        fiche: "js-bases",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "js_max",
        title: "maximum d'un tableau",
        difficulty: 2,
        task: "Écris une fonction `maximum(tab)` qui renvoie le plus grand nombre du tableau `tab` (avec une boucle ou reduce). Si `tab` est vide, renvoie undefined.",
        signature: "function maximum(tab)",
        starter: "function maximum(tab) {\n    // à toi\n}\n",
        harness: "function check(label, got, attendu) {\n    if (got === attendu) {\n        console.log('✓ ' + label);\n    } else {\n        console.log('✗ ' + label + ' = ' + got + ', attendu ' + attendu);\n        process.exit(84);\n    }\n}\n\ncheck('maximum([1, 5, 3])', maximum([1, 5, 3]), 5);\ncheck('maximum([-4, -1, -9])', maximum([-4, -1, -9]), -1);\ncheck('maximum([42])', maximum([42]), 42);\ncheck('maximum([])', maximum([]), undefined);\ncheck('maximum([7, 2])', maximum([7, 2]), 7);\n",
        hints: &[
            "Gère d'abord le tableau vide : if (tab.length === 0) return undefined;",
            "Boucle : let max = tab[0]; puis pour chaque élément, s'il est plus grand, remplace max.",
            "Version reduce : tab.reduce((a, b) => (b > a ? b : a)) — mais reduce sans valeur initiale plante sur un tableau vide, d'où la garde.",
        ],
        fiche: "js-arrays",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "js_inverser_objet",
        title: "inverser clés et valeurs d'un objet",
        difficulty: 3,
        task: "Écris une fonction `inverserObjet(obj)` qui renvoie un NOUVEL objet où les clés et les valeurs sont échangées : { a: 1 } devient { '1': 'a' } (les clés JS sont toujours des chaînes).",
        signature: "function inverserObjet(obj)",
        starter: "function inverserObjet(obj) {\n    // à toi\n}\n",
        harness: "function check(label, got, attendu) {\n    if (JSON.stringify(got) === JSON.stringify(attendu)) {\n        console.log('✓ ' + label);\n    } else {\n        console.log('✗ ' + label + ' = ' + JSON.stringify(got));\n        process.exit(84);\n    }\n}\n\ncheck('inverserObjet({a: 1, b: 2})', inverserObjet({a: 1, b: 2}), {'1': 'a', '2': 'b'});\ncheck('inverserObjet({nom: \"dupont\"})', inverserObjet({nom: 'dupont'}), {dupont: 'nom'});\ncheck('inverserObjet({})', inverserObjet({}), {});\n",
        hints: &[
            "Crée un objet vide res, puis parcours les clés de obj avec for (const cle in obj).",
            "res[obj[cle]] = cle; — la valeur devient la clé (convertie en chaîne), la clé devient la valeur.",
            "Renvoie res : c'est un NOUVEL objet, ne modifie pas obj.",
        ],
        fiche: "js-objets",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "sh_compter",
        title: "compter les fichiers d'un dossier",
        difficulty: 1,
        task: "Écris un script qui affiche le nombre de fichiers (pas les dossiers) du dossier courant.",
        signature: "#!/bin/bash — ton script affiche un nombre",
        starter: "#!/bin/bash\n# à toi\n",
        harness: "mkdir -p _t && cd _t && touch a b c.txt && mkdir sub && out=$(bash ../sol.sh); if [ \"$out\" = \"3\" ]; then echo '✓ 3 fichiers comptés'; exit 0; else echo '✗ attendu 3, obtenu '$out; exit 84; fi",
        hints: &[
            "ls te liste le contenu. wc -l compte les lignes.",
            "ls -p | grep -v / | wc -l",
        ],
        fiche: "commandes-texte",
        lang: "sh",
        solution_file: "",
    },
    Exo {
        id: "sh_somme",
        title: "somme des arguments",
        difficulty: 2,
        task: "Écris un script qui additionne tous ses arguments (des nombres) et affiche le total. Sans argument → affiche 0.",
        signature: "#!/bin/bash — ./exo.sh 1 2 3 → 6",
        starter: "#!/bin/bash\n# à toi\n",
        harness: "a=$(bash sol.sh 1 2 3); b=$(bash sol.sh); c=$(bash sol.sh 10 -5); ok=1; [ \"$a\" = \"6\" ] && echo '✓ 1+2+3=6' || { echo '✗ 1+2+3 → '$a; ok=0; }; [ \"$b\" = \"0\" ] && echo '✓ sans arg → 0' || { echo '✗ sans arg → '$b; ok=0; }; [ \"$c\" = \"5\" ] && echo '✓ 10-5=5' || { echo '✗ 10-5 → '$c; ok=0; }; [ $ok = 1 ] && exit 0 || exit 84",
        hints: &[
            "Les arguments sont $1, $2… $@ les contient tous.",
            "total=0; for n in \"$@\"; do total=$((total + n)); done; echo $total",
        ],
        fiche: "boucles-bash",
        lang: "sh",
        solution_file: "",
    },
    Exo {
        id: "js_dom_compteur",
        title: "un compteur avec le DOM",
        difficulty: 3,
        task: "Écris `incrementer()` qui lit la valeur de l'élément #compteur, l'incrémente de 1, et la réécrit. (Le harnais te fournit un `document` factice.)",
        signature: "function incrementer()",
        starter: "function incrementer() {\n    // à toi : utilise document.getElementById('compteur')\n}\n",
        harness: "const el = { textContent: '0' };\nconst document = { getElementById: (id) => id === 'compteur' ? el : null };\nincrementer();\nif (el.textContent === '1') { console.log('✓ 0 → 1'); } else { console.log('✗ compteur =', el.textContent); process.exit(84); }\nincrementer(); incrementer();\nif (el.textContent === '3') { console.log('✓ 3 incréments → 3'); } else { console.log('✗ après 3 clics:', el.textContent); process.exit(84); }\n",
        hints: &[
            "getElementById('compteur') te donne l'élément ; .textContent est son texte (une string).",
            "parseInt(el.textContent) + 1, puis réécris-le en string.",
        ],
        fiche: "js-dom",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "html_structure",
        title: "une page HTML bien structurée",
        difficulty: 1,
        task: "Crée une page HTML valide et sémantique : DOCTYPE, html lang=\"fr\", head avec charset et title, body avec header/nav/main/footer. Le fichier doit s'appeler index.html.",
        signature: "index.html sémantique",
        starter: "<!DOCTYPE html>\n<!-- à toi -->\n",
        harness: "f=$(ls *.html index.html 2>/dev/null | head -1); [ -z \"$f\" ] && { echo '✗ aucun .html trouvé'; exit 84; }; ok=1; grep -qi \"<!DOCTYPE\" \"$f\" && echo '✓ doctype' || { echo '✗ doctype manquant'; ok=0; }; grep -qi \"lang=\" \"$f\" && echo '✓ lang' || { echo '✗ lang manquant'; ok=0; }; grep -qi \"charset\" \"$f\" && echo '✓ charset' || { echo '✗ charset manquant'; ok=0; }; grep -qi \"<main\" \"$f\" && echo '✓ <main>' || { echo '✗ <main> manquant (sémantique)'; ok=0; }; grep -qi \"<footer\" \"$f\" && echo '✓ <footer>' || { echo '✗ <footer> manquant'; ok=0; }; [ $ok = 1 ] && exit 0 || exit 84",
        hints: &[
            "header/nav/main/footer remplacent des div génériques.",
            "Relis html-bases et html-semantique.",
        ],
        fiche: "html-semantique",
        lang: "html",
        solution_file: "index.html",
    },
    Exo {
        id: "css_centrer",
        title: "centrer un élément avec flexbox",
        difficulty: 2,
        task: "Crée un style.css qui centre le contenu : un sélecteur qui utilise display:flex, justify-content:center ET align-items:center.",
        signature: "style.css avec flexbox centré",
        starter: "/* à toi */\n",
        harness: "f=style.css; [ ! -f \"$f\" ] && f=$(ls *.css 2>/dev/null | head -1); [ -z \"$f\" ] && { echo '✗ aucun .css trouvé'; exit 84; }; ok=1; grep -Eq \"display: *flex\" \"$f\" && echo '✓ display:flex' || { echo '✗ display:flex manquant'; ok=0; }; grep -Eq \"justify-content: *center\" \"$f\" && echo '✓ justify-content:center' || { echo '✗ justify-content:center manquant'; ok=0; }; grep -Eq \"align-items: *center\" \"$f\" && echo '✓ align-items:center' || { echo '✗ align-items:center manquant'; ok=0; }; [ $ok = 1 ] && exit 0 || exit 84",
        hints: &[
            "Les 3 propriétés vont sur le CONTENEUR, pas l'élément.",
            "display:flex; justify-content:center; align-items:center;",
        ],
        fiche: "css-flexbox",
        lang: "html",
        solution_file: "style.css",
    },
    Exo {
        id: "my_str_isalpha",
        title: "vérifier qu'une chaîne n'est que des lettres",
        difficulty: 2,
        task: "Écris `int my_str_isalpha(char const *str)` qui renvoie 1 si `str` ne contient que des lettres (a-z, A-Z), 0 sinon. Une chaîne vide renvoie 1.",
        signature: "int my_str_isalpha(char const *str);",
        starter: "int my_str_isalpha(char const *str)\n{\n    (void)str;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_str_isalpha(char const *str);\n\nstatic int test(char *s, int attendu)\n{\n    int got = my_str_isalpha(s);\n    if (got != attendu) {\n        printf(\"✗ my_str_isalpha(\\\"%s\\\") = %d, attendu %d\\n\", s, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_str_isalpha(\\\"%s\\\") = %d\\n\", s, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", 1);\n    ko += test(\"abc\", 1);\n    ko += test(\"AbCdEf\", 1);\n    ko += test(\"abc123\", 0);\n    ko += test(\"hello world\", 0);\n    ko += test(\"42\", 0);\n    ko += test(\"piscine!\", 0);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Parcours la chaîne : dès qu'un caractère n'est PAS une lettre, renvoie 0.",
            "Un caractère c est une lettre si (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z').",
            "Si la boucle se termine sans mauvais caractère, return (1); — la chaîne vide passe donc toute seule.",
        ],
        fiche: "ctype",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_str_isnum",
        title: "vérifier qu'une chaîne n'est que des chiffres",
        difficulty: 2,
        task: "Écris `int my_str_isnum(char const *str)` qui renvoie 1 si `str` ne contient que des chiffres (0-9), 0 sinon. Une chaîne vide renvoie 1.",
        signature: "int my_str_isnum(char const *str);",
        starter: "int my_str_isnum(char const *str)\n{\n    (void)str;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_str_isnum(char const *str);\n\nstatic int test(char *s, int attendu)\n{\n    int got = my_str_isnum(s);\n    if (got != attendu) {\n        printf(\"✗ my_str_isnum(\\\"%s\\\") = %d, attendu %d\\n\", s, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_str_isnum(\\\"%s\\\") = %d\\n\", s, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", 1);\n    ko += test(\"42\", 1);\n    ko += test(\"0123456789\", 1);\n    ko += test(\"42abc\", 0);\n    ko += test(\"4 2\", 0);\n    ko += test(\"-42\", 0);\n    ko += test(\"abc\", 0);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Même principe que my_str_isalpha, avec les chiffres : dès qu'un caractère n'est pas entre '0' et '9', renvoie 0.",
            "if (!(str[i] >= '0' && str[i] <= '9')) return (0);",
            "Chaîne vide → la boucle ne tourne pas → le return (1); final s'applique. Attention : '-' n'est PAS un chiffre.",
        ],
        fiche: "ctype",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strlowcase",
        title: "mettre une chaîne en minuscules",
        difficulty: 2,
        task: "Écris `char *my_strlowcase(char *str)` qui met toutes les lettres de `str` en minuscules (modification en place) et renvoie `str`.",
        signature: "char *my_strlowcase(char *str);",
        starter: "char *my_strlowcase(char *str)\n{\n    // à toi\n    return (str);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strlowcase(char *str);\n\nstatic int test(char *src, char *attendu)\n{\n    char buf[64];\n    char *ret;\n\n    strcpy(buf, src);\n    ret = my_strlowcase(buf);\n    if (strcmp(buf, attendu) != 0 || ret != buf) {\n        printf(\"✗ my_strlowcase(\\\"%s\\\") → \\\"%s\\\", attendu \\\"%s\\\"\\n\", src, buf, attendu);\n        return (1);\n    }\n    printf(\"✓ my_strlowcase(\\\"%s\\\") → \\\"%s\\\"\\n\", src, buf);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", \"\");\n    ko += test(\"abc\", \"abc\");\n    ko += test(\"ABC\", \"abc\");\n    ko += test(\"HeLLo WoRLD\", \"hello world\");\n    ko += test(\"ABC123xYz\", \"abc123xyz\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Dans la table ASCII, les minuscules sont juste après les majuscules : 'a' - 'A' est le décalage.",
            "if (str[i] >= 'A' && str[i] <= 'Z') str[i] = str[i] + ('a' - 'A');",
            "On modifie str en place, puis return (str); — le harnais vérifie aussi le pointeur renvoyé.",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strupcase",
        title: "mettre une chaîne en majuscules",
        difficulty: 2,
        task: "Écris `char *my_strupcase(char *str)` qui met toutes les lettres de `str` en majuscules (modification en place) et renvoie `str`.",
        signature: "char *my_strupcase(char *str);",
        starter: "char *my_strupcase(char *str)\n{\n    // à toi\n    return (str);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strupcase(char *str);\n\nstatic int test(char *src, char *attendu)\n{\n    char buf[64];\n    char *ret;\n\n    strcpy(buf, src);\n    ret = my_strupcase(buf);\n    if (strcmp(buf, attendu) != 0 || ret != buf) {\n        printf(\"✗ my_strupcase(\\\"%s\\\") → \\\"%s\\\", attendu \\\"%s\\\"\\n\", src, buf, attendu);\n        return (1);\n    }\n    printf(\"✓ my_strupcase(\\\"%s\\\") → \\\"%s\\\"\\n\", src, buf);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", \"\");\n    ko += test(\"ABC\", \"ABC\");\n    ko += test(\"abc\", \"ABC\");\n    ko += test(\"HeLLo WoRLD\", \"HELLO WORLD\");\n    ko += test(\"abc123XyZ\", \"ABC123XYZ\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "C'est l'inverse de my_strlowcase : le décalage est 'a' - 'A', dans l'autre sens.",
            "if (str[i] >= 'a' && str[i] <= 'z') str[i] = str[i] - ('a' - 'A');",
            "On modifie str en place, puis return (str);",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strcapitalize",
        title: "mettre en capitale la première lettre de chaque mot",
        difficulty: 3,
        task: "Écris `char *my_strcapitalize(char *str)` qui met la première lettre de chaque mot en majuscule et tout le reste en minuscule. Un mot est une suite de lettres et/ou chiffres ; tout autre caractère est un séparateur. Modifie `str` en place et renvoie-la.",
        signature: "char *my_strcapitalize(char *str);",
        starter: "char *my_strcapitalize(char *str)\n{\n    // à toi\n    return (str);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strcapitalize(char *str);\n\nstatic int test(char *src, char *attendu)\n{\n    char buf[128];\n    char *ret;\n\n    strcpy(buf, src);\n    ret = my_strcapitalize(buf);\n    if (strcmp(buf, attendu) != 0 || ret != buf) {\n        printf(\"✗ my_strcapitalize(\\\"%s\\\") → \\\"%s\\\", attendu \\\"%s\\\"\\n\", src, buf, attendu);\n        return (1);\n    }\n    printf(\"✓ my_strcapitalize(\\\"%s\\\") → \\\"%s\\\"\\n\", src, buf);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", \"\");\n    ko += test(\"hello world\", \"Hello World\");\n    ko += test(\"HELLO wOrLd\", \"Hello World\");\n    ko += test(\"hi, how are you?\", \"Hi, How Are You?\");\n    ko += test(\"42hello world\", \"42hello World\");\n    ko += test(\"salut, comment tu vas ? 42mots\", \"Salut, Comment Tu Vas ? 42mots\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Deux passes : d'abord TOUT en minuscules (comme my_strlowcase), puis remets une majuscule au début de chaque mot.",
            "Un caractère commence un mot s'il est en position 0, ou si le caractère PRÉCÉDENT n'est ni une lettre ni un chiffre.",
            "Écris un helper is_alnum(c) (lettre ou chiffre). Puis pour i >= 1 : if (!is_alnum(str[i - 1]) && str[i] entre 'a' et 'z') → majuscule. Attention : dans \"42hello\", le 'h' suit un chiffre, donc il reste en minuscule.",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strncpy",
        title: "copier une chaîne sur n caractères",
        difficulty: 2,
        task: "Écris `char *my_strncpy(char *dest, char const *src, int n)` qui copie au plus `n` caractères de `src` dans `dest` : si `src` fait moins de `n` caractères, complète avec des '\\0' (padding). Renvoie `dest`.",
        signature: "char *my_strncpy(char *dest, char const *src, int n);",
        starter: "char *my_strncpy(char *dest, char const *src, int n)\n{\n    (void)src;\n    (void)n;\n    // à toi\n    return (dest);\n}\n",
        harness: "#include <stdio.h>\n#include <string.h>\nchar *my_strncpy(char *dest, char const *src, int n);\n\nstatic int test(char *src, int n)\n{\n    char a[64];\n    char b[64];\n    char *ret;\n\n    memset(a, 'X', 64);\n    memset(b, 'X', 64);\n    strncpy(a, src, (size_t)n);\n    ret = my_strncpy(b, src, n);\n    if (memcmp(a, b, 64) != 0 || ret != b) {\n        printf(\"✗ my_strncpy(\\\"%s\\\", %d) : copie incorrecte (padding '\\\\0' compris)\\n\", src, n);\n        return (1);\n    }\n    printf(\"✓ my_strncpy(\\\"%s\\\", %d)\\n\", src, n);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"hello\", 8);\n    ko += test(\"hello\", 3);\n    ko += test(\"hello\", 5);\n    ko += test(\"\", 4);\n    ko += test(\"epitech piscine\", 7);\n    ko += test(\"abc\", 0);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Deux boucles : d'abord copier src tant que i < n ET src[i] != '\\0'…",
            "…puis compléter avec des '\\0' tant que i < n (c'est le padding, comme le vrai strncpy).",
            "while (i < n && src[i] != '\\0') { dest[i] = src[i]; i++; } puis while (i < n) { dest[i] = '\\0'; i++; } return (dest);",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "js_events",
        title: "réagir aux événements",
        difficulty: 2,
        task: "Écris `brancher(bouton, compteur)` qui, à chaque 'click' du bouton, incrémente le texte du compteur. (Le harnais fournit des éléments factices avec addEventListener et un .click() simulé.)",
        signature: "function brancher(bouton, compteur)",
        starter: "function brancher(bouton, compteur) {\n    // à toi\n}\n",
        harness: "function fakeEl() { const h = {}; return { textContent: '0', addEventListener: (ev, fn) => { h[ev] = fn; }, click: () => h.click && h.click() }; }\nconst btn = fakeEl(); const cpt = fakeEl();\nbrancher(btn, cpt);\nbtn.click();\nif (cpt.textContent === '1') { console.log('✓ 1 clic → 1'); } else { console.log('✗ après 1 clic:', cpt.textContent); process.exit(84); }\nbtn.click(); btn.click();\nif (cpt.textContent === '3') { console.log('✓ 3 clics → 3'); } else { console.log('✗ après 3 clics:', cpt.textContent); process.exit(84); }\n",
        hints: &[
            "addEventListener('click', callback) : la callback s'exécute à chaque clic.",
            "Dans la callback : lis cpt.textContent, +1, réécris.",
        ],
        fiche: "js-evenements",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "js_async",
        title: "attendre une promesse",
        difficulty: 3,
        task: "Écris `charger()` async qui attend une promesse `fakeFetch()` (fournie) et renvoie sa valeur. Utilise await.",
        signature: "async function charger()",
        starter: "async function charger() {\n    // à toi\n}\n",
        harness: "function fakeFetch() { return Promise.resolve('données'); }\n(async () => {\n    const v = await charger();\n    if (v === 'données') { console.log('✓ charger renvoie la valeur de la promesse'); } else { console.log('✗ charger →', v); process.exit(84); }\n    if (charger() instanceof Promise) { console.log('✓ charger est async'); } else { process.exit(84); }\n})();\n",
        hints: &[
            "async/await : await attend la promesse et donne sa valeur.",
            "return await fakeFetch();",
        ],
        fiche: "js-async",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "my_list_reverse",
        title: "inverser une liste chaînée en place",
        difficulty: 4,
        task: "Écris `t_list *my_list_reverse(t_list *list)` qui inverse la liste chaînée EN PLACE : les pointeurs `next` sont réorientés, AUCUN nœud n'est créé ni libéré (pas de malloc). Renvoie la nouvelle tête (l'ancien dernier nœud). Une liste vide renvoie NULL.",
        signature: "t_list *my_list_reverse(t_list *list);",
        starter: "#include <stddef.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nt_list *my_list_reverse(t_list *list)\n{\n    (void)list;\n    // à toi\n    return (NULL);\n}\n",
        harness: "#include <stdio.h>\n#include <stdlib.h>\n#include <stddef.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nt_list *my_list_reverse(t_list *list);\n\nstatic t_list *node(void *data)\n{\n    t_list *n = malloc(sizeof(t_list));\n\n    n->data = data;\n    n->next = NULL;\n    return (n);\n}\n\nstatic void free_list(t_list *l)\n{\n    t_list *tmp;\n\n    while (l != NULL) {\n        tmp = l;\n        l = l->next;\n        free(tmp);\n    }\n}\n\nint main(void)\n{\n    int a = 1;\n    int b = 2;\n    int c = 3;\n    t_list *n1;\n    t_list *n2;\n    t_list *n3;\n    t_list *ret;\n    int ko = 0;\n\n    n1 = node(&a);\n    n2 = node(&b);\n    n3 = node(&c);\n    n1->next = n2;\n    n2->next = n3;\n    ret = my_list_reverse(n1);\n    if (ret != n3 || n3->next != n2 || n2->next != n1 || n1->next != NULL) {\n        printf(\"✗ 1→2→3 inversée : attendu 3→2→1 (mêmes nœuds, en place)\\n\");\n        ko = 1;\n    } else {\n        printf(\"✓ 1→2→3 inversée → 3→2→1 (en place)\\n\");\n    }\n    free_list(ret);\n    n1 = node(&a);\n    ret = my_list_reverse(n1);\n    if (ret != n1 || n1->next != NULL) {\n        printf(\"✗ liste à 1 nœud : doit rester identique\\n\");\n        ko = 1;\n    } else {\n        printf(\"✓ liste à 1 nœud inchangée\\n\");\n    }\n    free_list(ret);\n    if (my_list_reverse(NULL) != NULL) {\n        printf(\"✗ liste vide : attendu NULL\\n\");\n        ko = 1;\n    } else {\n        printf(\"✓ liste vide → NULL\\n\");\n    }\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Trois pointeurs : prev (NULL au départ), cur (le nœud en cours), next (pour ne pas perdre la suite).",
            "Dans la boucle : next = cur->next; puis cur->next = prev; puis prev = cur; puis cur = next;",
            "Quand cur vaut NULL, la nouvelle tête est prev : return (prev);",
        ],
        fiche: "listes-chainees",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_list_remove_if",
        title: "supprimer des nœuds selon un prédicat",
        difficulty: 4,
        task: "Écris `void my_list_remove_if(t_list **list, int (*pred)(void *data))` qui supprime de la liste tous les nœuds pour lesquels `pred(data)` renvoie non nul. Chaque nœud supprimé est libéré avec free. La liste est modifiée en place : `*list` peut changer (la tête peut être supprimée).",
        signature: "void my_list_remove_if(t_list **list, int (*pred)(void *data));",
        starter: "#include <stdlib.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nvoid my_list_remove_if(t_list **list, int (*pred)(void *data))\n{\n    (void)list;\n    (void)pred;\n    // à toi\n}\n",
        harness: "#include <stdio.h>\n#include <stdlib.h>\n#include <stddef.h>\n\ntypedef struct s_list {\n    void *data;\n    struct s_list *next;\n} t_list;\n\nvoid my_list_remove_if(t_list **list, int (*pred)(void *data));\n\nstatic int est_pair(void *data)\n{\n    return (*(int *)data % 2 == 0);\n}\n\nstatic int tout(void *data)\n{\n    (void)data;\n    return (1);\n}\n\nstatic t_list *build(int *vals, int n)\n{\n    t_list *head = NULL;\n    t_list *node;\n    int i;\n\n    i = n - 1;\n    while (i >= 0) {\n        node = malloc(sizeof(t_list));\n        node->data = &vals[i];\n        node->next = head;\n        head = node;\n        i--;\n    }\n    return (head);\n}\n\nstatic void free_list(t_list *l)\n{\n    t_list *tmp;\n\n    while (l != NULL) {\n        tmp = l;\n        l = l->next;\n        free(tmp);\n    }\n}\n\nstatic int check(t_list *head, int attendu[], int n)\n{\n    int i = 0;\n\n    while (i < n && head != NULL) {\n        if (*(int *)head->data != attendu[i])\n            return (0);\n        head = head->next;\n        i++;\n    }\n    return (i == n && head == NULL);\n}\n\nint main(void)\n{\n    int v1[5] = {1, 2, 3, 4, 5};\n    int v2[3] = {2, 4, 1};\n    int v3[2] = {1, 2};\n    int e1[3] = {1, 3, 5};\n    int e2[1] = {1};\n    t_list *l;\n    int ko = 0;\n\n    l = build(v1, 5);\n    my_list_remove_if(&l, &est_pair);\n    if (check(l, e1, 3)) {\n        printf(\"✓ 1→2→3→4→5 moins les pairs → 1→3→5\\n\");\n    } else {\n        printf(\"✗ suppression des pairs dans 1→2→3→4→5 : attendu 1→3→5\\n\");\n        ko = 1;\n    }\n    free_list(l);\n    l = build(v2, 3);\n    my_list_remove_if(&l, &est_pair);\n    if (check(l, e2, 1)) {\n        printf(\"✓ 2→4→1 moins les pairs → 1 (la tête était supprimée)\\n\");\n    } else {\n        printf(\"✗ suppression en tête de 2→4→1 : attendu 1\\n\");\n        ko = 1;\n    }\n    free_list(l);\n    l = build(v3, 2);\n    my_list_remove_if(&l, &tout);\n    if (l == NULL) {\n        printf(\"✓ tout supprimer → liste vide\\n\");\n    } else {\n        printf(\"✗ tout supprimer : la liste devrait être vide\\n\");\n        ko = 1;\n    }\n    free_list(l);\n    l = NULL;\n    my_list_remove_if(&l, &est_pair);\n    if (l == NULL) {\n        printf(\"✓ liste vide : pas de crash\\n\");\n    } else {\n        printf(\"✗ liste vide : *list doit rester NULL\\n\");\n        ko = 1;\n    }\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Parcours avec un pointeur cur ; quand pred(cur->data) est vrai, détache le nœud de la liste, avance, puis free-le.",
            "Le cas délicat, c'est la tête : si le 1er nœud matche, c'est *list qui doit avancer. Un t_list ** courant (ou un pointeur sur le lien précédent) unifie les cas.",
            "t_list *tmp; si suppression : tmp = cur; cur = cur->next; *lien = cur; free(tmp); sinon : lien = &cur->next; cur = cur->next;",
        ],
        fiche: "listes-chainees",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_str_islower",
        title: "vérifier qu'une chaîne n'est que des minuscules",
        difficulty: 2,
        task: "Écris `int my_str_islower(char const *str)` qui renvoie 1 si `str` ne contient que des lettres minuscules (a-z), 0 sinon. Une chaîne vide renvoie 1.",
        signature: "int my_str_islower(char const *str);",
        starter: "int my_str_islower(char const *str)\n{\n    (void)str;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_str_islower(char const *str);\n\nstatic int test(char *s, int attendu)\n{\n    int got = my_str_islower(s);\n    if (got != attendu) {\n        printf(\"✗ my_str_islower(\\\"%s\\\") = %d, attendu %d\\n\", s, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_str_islower(\\\"%s\\\") = %d\\n\", s, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\", 1);\n    ko += test(\"abc\", 1);\n    ko += test(\"abcdefghijklmnopqrstuvwxyz\", 1);\n    ko += test(\"z\", 1);\n    ko += test(\"abC\", 0);\n    ko += test(\"ABC\", 0);\n    ko += test(\"abc1\", 0);\n    ko += test(\"hello world\", 0);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Parcours la chaîne : dès qu'un caractère n'est PAS entre 'a' et 'z', renvoie 0.",
            "if (!(str[i] >= 'a' && str[i] <= 'z')) return (0);",
            "Chaîne vide → la boucle ne tourne pas → le return (1); final s'applique. Les majuscules, chiffres et espaces renvoient 0.",
        ],
        fiche: "chaines-caracteres",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_power_iterative",
        title: "puissance avec une boucle",
        difficulty: 2,
        task: "Écris `int my_power_iterative(int nb, int p)` qui renvoie `nb` élevé à la puissance `p` de façon ITÉRATIVE (une boucle, SANS appel récursif). Si `p` est négatif, renvoie 0. C'est le pendant itératif de my_compute_power_rec : mêmes résultats, aucune pile d'appels.",
        signature: "int my_power_iterative(int nb, int p);",
        starter: "int my_power_iterative(int nb, int p)\n{\n    (void)nb;\n    (void)p;\n    // à toi\n    return (0);\n}\n",
        harness: "#include <stdio.h>\nint my_power_iterative(int nb, int p);\n\nstatic int test(int nb, int p, int attendu)\n{\n    int got = my_power_iterative(nb, p);\n    if (got != attendu) {\n        printf(\"✗ my_power_iterative(%d, %d) = %d, attendu %d\\n\", nb, p, got, attendu);\n        return (1);\n    }\n    printf(\"✓ my_power_iterative(%d, %d) = %d\\n\", nb, p, got);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(2, 0, 1);\n    ko += test(2, 3, 8);\n    ko += test(5, -1, 0);\n    ko += test(0, 0, 1);\n    ko += test(3, 4, 81);\n    ko += test(-2, 3, -8);\n    ko += test(-2, 2, 4);\n    ko += test(10, 5, 100000);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Initialise un accumulateur res à 1, puis multiplie-le par nb, p fois.",
            "Gardes d'abord : if (p < 0) return (0); — et p == 0 donne naturellement 1 (la boucle ne tourne pas).",
            "while (p > 0) { res = res * nb; p--; } return (res);",
        ],
        fiche: "boucles",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_strdup",
        title: "dupliquer une chaîne avec malloc",
        difficulty: 3,
        task: "Écris `char *my_strdup(char const *src)` qui renvoie une COPIE de `src` allouée avec malloc (contenu identique, adresse différente, '\\0' compris). Renvoie NULL si l'allocation échoue. L'appelant libère la copie avec free.",
        signature: "char *my_strdup(char const *src);",
        starter: "#include <stdlib.h>\n\nchar *my_strdup(char const *src)\n{\n    (void)src;\n    // à toi\n    return (NULL);\n}\n",
        harness: "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\nchar *my_strdup(char const *src);\n\nstatic int test(char const *src)\n{\n    char buf[64];\n    char *dup;\n\n    strcpy(buf, src);\n    dup = my_strdup(buf);\n    if (dup == NULL) {\n        printf(\"✗ my_strdup(\\\"%s\\\") → NULL (malloc raté ?)\\n\", src);\n        return (1);\n    }\n    if (strcmp(dup, buf) != 0 || dup == buf) {\n        printf(\"✗ my_strdup(\\\"%s\\\") : contenu différent ou même adresse\\n\", src);\n        free(dup);\n        return (1);\n    }\n    if (buf[0] != '\\0') {\n        dup[0] = 'X';\n        if (buf[0] == 'X') {\n            printf(\"✗ my_strdup(\\\"%s\\\") : modifier la copie touche l'original\\n\", src);\n            free(dup);\n            return (1);\n        }\n    }\n    printf(\"✓ my_strdup(\\\"%s\\\") → copie indépendante\\n\", src);\n    free(dup);\n    return (0);\n}\n\nint main(void)\n{\n    int ko = 0;\n    ko += test(\"\");\n    ko += test(\"a\");\n    ko += test(\"hello world\");\n    ko += test(\"piscine epitech 42\");\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Mesure d'abord la longueur de src (comme my_strlen), puis malloc(longueur + 1) : le +1 c'est pour le '\\0'.",
            "Vérifie TOUJOURS le retour de malloc : if (dup == NULL) return (NULL);",
            "Recopie caractère par caractère, '\\0' INCLUS (boucle avec i <= len, ou copie puis dup[len] = '\\0';).",
        ],
        fiche: "malloc",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "my_sort_int_tab",
        title: "trier un tableau d'entiers",
        difficulty: 3,
        task: "Écris `void my_sort_int_tab(int *tab, int size)` qui trie le tableau `tab` de `size` entiers par ordre CROISSANT, en place. Un bubble sort suffit amplement.",
        signature: "void my_sort_int_tab(int *tab, int size);",
        starter: "void my_sort_int_tab(int *tab, int size)\n{\n    (void)tab;\n    (void)size;\n    // à toi\n}\n",
        harness: "#include <stdio.h>\n\nvoid my_sort_int_tab(int *tab, int size);\n\nstatic int test(int *tab, int *attendu, int size)\n{\n    int i;\n    int ok = 1;\n\n    my_sort_int_tab(tab, size);\n    i = 0;\n    while (i < size) {\n        if (tab[i] != attendu[i])\n            ok = 0;\n        i++;\n    }\n    if (!ok) {\n        printf(\"✗ tableau de %d élément(s) mal trié\\n\", size);\n        return (1);\n    }\n    printf(\"✓ tableau de %d élément(s) trié par ordre croissant\\n\", size);\n    return (0);\n}\n\nint main(void)\n{\n    int t1[6] = {5, 1, 4, 2, 8, 0};\n    int a1[6] = {0, 1, 2, 4, 5, 8};\n    int t2[1] = {42};\n    int a2[1] = {42};\n    int t3[4] = {4, 3, 2, 1};\n    int a3[4] = {1, 2, 3, 4};\n    int t4[5] = {-3, 5, -10, 0, 7};\n    int a4[5] = {-10, -3, 0, 5, 7};\n    int t5[5] = {1, 2, 3, 4, 5};\n    int a5[5] = {1, 2, 3, 4, 5};\n    int ko = 0;\n\n    ko += test(t1, a1, 6);\n    ko += test(t2, a2, 1);\n    ko += test(t3, a3, 4);\n    ko += test(t4, a4, 5);\n    ko += test(t5, a5, 5);\n    return (ko ? 84 : 0);\n}\n",
        hints: &[
            "Bubble sort : deux boucles imbriquées ; si deux voisins sont dans le mauvais ordre, échange-les.",
            "L'échange, c'est my_swap sur tab[j] et tab[j + 1] : une variable tmp pour ne pas écraser une valeur.",
            "for i de 0 à size-2, for j de 0 à size-2-i : if (tab[j] > tab[j + 1]) → échange. Après chaque passe i, les i plus grands sont en place à la fin.",
        ],
        fiche: "tableaux",
        lang: "c",
        solution_file: "",
    },
    Exo {
        id: "js_formulaire",
        title: "valider un formulaire",
        difficulty: 3,
        task: "Écris `valider(email, age)` qui renvoie true si l'email contient un '@' ET un '.' ET que l'âge est un nombre >= 18.",
        signature: "function valider(email, age)",
        starter: "function valider(email, age) {\n    // à toi\n}\n",
        harness: "if (valider('a@b.co', 20) === true) { console.log('✓ valide'); } else { console.log('✗ cas valide rejeté'); process.exit(84); }\nif (valider('pasemail', 20) === false) { console.log('✗ email sans @ rejeté'); } else { process.exit(84); }\nif (valider('a@b.co', 15) === false) { console.log('✗ mineur rejeté'); } else { process.exit(84); }\nif (valider('a@b.co', 'abc') === false) { console.log('✗ âge non-nombre rejeté'); } else { process.exit(84); }\n",
        hints: &[
            "email.includes('@') && email.includes('.'), et Number(age) >= 18.",
            "Vérifie typeof ou isNaN pour l'âge.",
        ],
        fiche: "js-formulaires",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "js_localstorage",
        title: "sauvegarder dans le navigateur",
        difficulty: 2,
        task: "Écris `sauver(cle, valeur)` et `charger(cle)` qui utilisent localStorage (JSON pour les objets). (Le harnais fournit un localStorage factice.)",
        signature: "function sauver(cle, valeur) / function charger(cle)",
        starter: "function sauver(cle, valeur) {\n    // à toi\n}\n\nfunction charger(cle) {\n    // à toi\n}\n",
        harness: "const store = {};\nconst localStorage = { setItem: (k, v) => { store[k] = String(v); }, getItem: (k) => store[k] ?? null };\nsauver('user', { nom: 'sam' });\nconst r = charger('user');\nif (r && r.nom === 'sam') { console.log('✓ objet sauvé et rechargé'); } else { console.log('✗ rechargé:', r); process.exit(84); }\nif (charger('inexistant') === null) { console.log('✗ clé absente → null'); } else { process.exit(84); }\n",
        hints: &[
            "JSON.stringify pour sauver, JSON.parse pour recharger.",
            "getItem renvoie null si la clé n'existe pas.",
        ],
        fiche: "js-localstorage",
        lang: "js",
        solution_file: "",
    },
    Exo {
        id: "js_todo",
        title: "une todo list complète (projet guidé)",
        difficulty: 4,
        task: "Construis une todo list : `creerTodo()` renvoie une liste vide, `ajouterTodo(liste, texte)` ajoute {texte, fait:false}, `terminerTodo(liste, i)` marque fait, `compterRestants(liste)` compte les non-faits.",
        signature: "4 fonctions : creerTodo / ajouterTodo / terminerTodo / compterRestants",
        starter: "function creerTodo() {\n    // à toi\n}\n\nfunction ajouterTodo(liste, texte) {\n    // à toi\n}\n\nfunction terminerTodo(liste, i) {\n    // à toi\n}\n\nfunction compterRestants(liste) {\n    // à toi\n}\n",
        harness: "const l = creerTodo();\nif (!Array.isArray(l) || l.length === 0) { console.log('✓ liste vide au départ'); } else { process.exit(84); }\najouterTodo(l, 'réviser'); ajouterTodo(l, 'coder');\nif (l.length === 2 && l[0].texte === 'réviser' && l[0].fait === false) { console.log('✓ ajout'); } else { console.log('✗ ajout:', l); process.exit(84); }\nterminerTodo(l, 0);\nif (l[0].fait === true) { console.log('✓ terminer'); } else { process.exit(84); }\nif (compterRestants(l) === 1) { console.log('✓ 1 restant'); } else { console.log('✗ restants:', compterRestants(l)); process.exit(84); }\n",
        hints: &[
            "Une todo = un objet {texte, fait}. La liste = un tableau.",
            "ajouterTodo : liste.push({texte, fait:false}). terminerTodo : liste[i].fait = true.",
            "compterRestants : filter(t => !t.fait).length",
        ],
        fiche: "js-arrays",
        lang: "js",
        solution_file: "",
    },
];

// les exos JS s'exécutent avec node (la solution est injectée dans globalThis)
pub fn is_node_available() -> bool {
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

impl Exo {
    /// Nom du fichier de solution (par défaut <id>.c).
    pub fn solution_name(&self) -> String {
        if self.solution_file.is_empty() {
            format!("{}.c", self.id)
        } else {
            self.solution_file.to_string()
        }
    }
}

pub fn by_id(id: &str) -> Option<&'static Exo> {
    EXOS.iter().find(|e| e.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harnais_des_exos_compilent() {
        for exo in EXOS.iter().filter(|e| e.lang == "c") {
            // on compile le starter + le harnais : le starter doit au moins compiler
            let dir = std::env::temp_dir().join(format!("cman-exo-test-{}", exo.id));
            std::fs::create_dir_all(&dir).unwrap();
            let sol = dir.join(format!("{}.c", exo.id));
            let test = dir.join("test.c");
            std::fs::write(&sol, exo.starter).unwrap();
            std::fs::write(&test, exo.harness).unwrap();
            let out = std::process::Command::new("gcc")
                .args(["-Wall", "-Wextra", "-std=c99", "-o"])
                .arg(dir.join("t"))
                .arg(&sol)
                .arg(&test)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "le harnais de {} doit compiler avec le starter: {}",
                exo.id,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
