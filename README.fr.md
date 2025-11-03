# Finder

`finder` est un outil en ligne de commande simple et rapide écrit en Rust pour rechercher une chaîne de caractères dans des fichiers et des répertoires.

## Fonctionnalités

- Recherche d'une seule chaîne de caractères ou de plusieurs chaînes à partir d'un fichier.
- Recherche récursive dans les répertoires.
- Affichage du numéro de ligne, du motif trouvé et du contenu de la ligne correspondante.
- Barre de progression pendant la recherche.
- Statistiques de recherche (nombre de correspondances, temps écoulé, etc.).
- Traitement parallèle pour des recherches plus rapides.
- Sortie colorée pour une meilleure lisibilité.
- Gère correctement les fins de ligne Windows (CRLF) et Unix (LF).

## Dépendances

Ce projet utilise les dépendances suivantes (telles que définies dans `Cargo.toml`) :

- `clap` (version `4.5.51`) : Pour l'analyse des arguments de la ligne de commande.
- `indicatif` (version `0.18.2`) : Pour afficher une barre de progression.
- `rayon` (version `1.11.0`) : Pour le traitement parallèle.
- `colored` (version `3.0.0`) : Pour colorer la sortie du terminal.
- `encoding_rs` (version `0.8.35`) : Pour la gestion des encodages de fichiers.
- `encoding_rs_io` (version `0.1.7`) : Pour la lecture de fichiers avec différents encodages.
- `ignore` (version `0.4.25`) : Pour ignorer les fichiers et répertoires.
- `regex` (version `1.12.2`) : Pour la recherche avec des expressions régulières.
- `tempfile` (version `3.23.0`) : Pour la création de fichiers et répertoires temporaires dans les tests.

## Installation

### Prérequis

Assurez-vous d'avoir Rust et Cargo d'installés sur votre système. Vous pouvez les installer en suivant les instructions sur le site officiel de Rust : [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compilation pour Linux (depuis Linux/macOS)
1.  Clonez ce dépôt :
    ```sh
    git clone https://github.com/cederig/finder.git
    cd finder
    ```
2.  Compilez le projet :
    ```sh
    cargo build --release
    ```
    L'exécutable se trouvera dans `target/release/finder`.

### Compilation pour Windows (depuis Linux/macOS)

Pour compiler ce projet pour Windows à partir d'un autre système d'exploitation (comme Linux ou macOS), vous pouvez utiliser la compilation croisée. Vous aurez besoin de la cible Rust pour Windows.

1.  Ajoutez la cible Windows à votre installation Rust :
    ```sh
    rustup target add x86_64-pc-windows-gnu
    ```

2.  Compilez le projet pour la cible Windows :
    ```sh
    cargo build --release --target=x86_64-pc-windows-gnu
    ```

L'exécutable pour Windows se trouvera dans `target/x86_64-pc-windows-gnu/release/finder.exe`.

### Compilation pour macOS (depuis Linux/macOS)

Pour compiler ce projet pour macOS à partir d'un autre système d'exploitation (comme Linux ou macOS), vous pouvez utiliser la compilation croisée. Vous aurez besoin de la cible Rust pour macOS.

1.  Ajoutez la cible macOS à votre installation Rust (choisissez la bonne architecture) :
    *   Pour les Mac Intel (x86_64) :
        ```sh
        rustup target add x86_64-apple-darwin
        ```
    *   Pour les Mac Apple Silicon (aarch64) :
        ```sh
        rustup target add aarch64-apple-darwin
        ```

2.  Compilez le projet pour la cible macOS (choisissez la bonne architecture) :
    *   Pour les Mac Intel :
        ```sh
        cargo build --release --target=x86_64-apple-darwin
        ```
    *   Pour les Mac Apple Silicon :
        ```sh
        cargo build --release --target=aarch64-apple-darwin
        ```

L'exécutable pour macOS se trouvera dans `target/<votre_cible_mac>/release/finder` (par exemple, `target/x86_64-apple-darwin/release/finder`).

## Utilisation

```bash
finder [OPTIONS] <PATHS>... -p <PATTERN>
finder [OPTIONS] <PATHS>... -f <FILE>
```

### Arguments

-   `<PATHS>...` : Un ou plusieurs chemins de fichiers ou de répertoires dans lesquels rechercher.

### Options

-   `-p`, `--pattern <PATTERN>` : La chaîne de caractères à rechercher. Mutuellement exclusif avec `-f`.
-   `-f`, `--input-file <FILE>` : Recherche les motifs depuis un fichier (un par ligne). Mutuellement exclusif avec `-p`.
-   `-i`, `--ignore-case` : Effectue une recherche insensible à la casse.
-   `-o`, `--output <FILE>` : Exporte les résultats vers le fichier spécifié au lieu de les afficher sur la console.
-   `-s`, `--stat` : Affiche des statistiques détaillées après la recherche.
-   `-h`, `--help` : Affiche l'aide.
-   `-V`, `--version` : Affiche la version de l'outil.

## Exemples

-   Rechercher "hello" dans un fichier (le mot "hello" sera surligné en rouge) :
    ```sh
    ./finder mon_fichier.txt -p "hello"
    ```

-   Rechercher "error" dans plusieurs fichiers :
    ```sh
    ./finder fichier1.log fichier2.log -p "error"
    ```

-   Rechercher tous les motifs de `patterns.txt` dans un répertoire entier :
    ```sh
    ./finder ./mon_projet/ -f patterns.txt
    ```

-   Rechercher avec des statistiques :
    ```sh
    ./finder ./docs/ --stat -p "important"
    ```

## Format de sortie

Le format de sortie est le suivant :
`chemin/vers/le/fichier:numero_de_ligne:motif_trouve:contenu_de_la_ligne_surlignee`

## Ignorer des fichiers

`finder` respecte automatiquement les règles définies dans les fichiers `.gitignore` et `.ignore`.

## Tests

Ce projet inclut des tests unitaires; pour les exécuter, utilisez la commande suivante à la racine du projet :

```sh
cargo test
```

Cette commande compile le programme en mode test et exécute toutes les fonctions de test.
