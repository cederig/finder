# Finder

`finder` est un outil en ligne de commande simple et rapide écrit en Rust pour rechercher une chaîne de caractères dans des fichiers et des répertoires.

## Fonctionnalités

- Recherche de chaînes de caractères dans un ou plusieurs fichiers.
- Recherche récursive dans les répertoires.
- Affichage du numéro de ligne et du contenu de la ligne correspondante.
- Barre de progression pendant la recherche.
- Statistiques de recherche (nombre de correspondances, temps écoulé, etc.).
- Traitement parallèle pour des recherches plus rapides.
- Sortie colorée pour une meilleure lisibilité.

## Dépendances

Ce projet utilise les dépendances suivantes (telles que définies dans `Cargo.toml`) :

- `clap` (version `4.5.41`) : Pour l'analyse des arguments de la ligne de commande.
- `indicatif` (version `0.18.0`) : Pour afficher une barre de progression.
- `rayon` (version `1.10.0`) : Pour le traitement parallèle.
- `colored` (version `2.1.0`) : Pour colorer la sortie du terminal.
- `encoding_rs` (version `0.8.35`) : Pour la gestion des encodages de fichiers.
- `encoding_rs_io` (version `0.1.7`) : Pour la lecture de fichiers avec différents encodages.
- `ignore` (version `0.4.23`) : Pour ignorer les fichiers et répertoires.
- `regex` (version `1.11.1`) : Pour la recherche avec des expressions régulières.
- `tempfile` (version `3.20.0`) : Pour la création de fichiers et répertoires temporaires dans les tests.

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

La syntaxe de base est la suivante :

```bash
./finder [OPTIONS] <PATTERN> <PATHS>...
```

### Arguments

-   `<PATTERN>` : La chaîne de caractères à rechercher.
-   `<PATHS>...` : Un ou plusieurs chemins de fichiers ou de répertoires dans lesquels rechercher.

### Options

-   `-i`, `--ignore-case` : Effectue une recherche insensible à la casse.
-   `-o`, `--output <FILE>` : Exporte les résultats vers le fichier spécifié au lieu de les afficher sur la console.
-   `-s`, `--stat` : Affiche des statistiques détaillées après la recherche.
-   `-h`, `--help` : Affiche l'aide.
-   `-V`, `--version` : Affiche la version de l'outil.

## Exemples

-   Rechercher "hello" dans un fichier (le mot "hello" sera surligné en rouge) :
    ```sh
    ./finder "hello" mon_fichier.txt
    ```

-   Rechercher "error" dans plusieurs fichiers :
    ```sh
    ./finder "error" fichier1.log fichier2.log
    ```

-   Rechercher "TODO" dans un répertoire entier :
    ```sh
    ./finder "TODO" ./mon_projet/
    ```

-   Rechercher avec des statistiques :
    ```sh
    ./finder --stat "important" ./docs/
    ```
## Ignorer des fichiers

`finder` respecte automatiquement les règles définies dans les fichiers `.gitignore` et `.ignore`. Cela signifie que les fichiers et répertoires qui sont généralement ignorés dans un projet (comme `target/`, `node_modules/`, etc.) seront automatiquement exclus de la recherche. Vous pouvez personnaliser ce comportement en créant vos propres fichiers `.ignore` dans votre projet.


## Tests

Ce projet inclut des tests unitaires; pour les exécuter, utilisez la commande suivante à la racine du projet :

```sh
cargo test
```

Cette commande compile le programme en mode test et exécute toutes les fonctions de test.