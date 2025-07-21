# Finder

`finder` est un outil en ligne de commande simple et rapide pour rechercher une chaîne de caractères dans des fichiers et des répertoires.

## Fonctionnalités

- Recherche de chaînes de caractères dans un ou plusieurs fichiers.
- Recherche récursive dans les répertoires.
- Affichage du numéro de ligne et du contenu de la ligne correspondante.
- Barre de progression pendant la recherche.
- Statistiques de recherche (nombre de correspondances, temps écoulé, etc.).
- Traitement parallèle pour des recherches plus rapides.
- Sortie colorée pour une meilleure lisibilité.

## Dépendances

Ce projet utilise les dépendances Rust suivantes :

- `clap` (v4.5.7) : Pour l'analyse des arguments de la ligne de commande.
- `indicatif` (v0.18.0) : Pour afficher une barre de progression.
- `rayon` (v1.10.0) : Pour le traitement parallèle.
- `colored` (v2.1.0) : Pour colorer la sortie du terminal.
- `encoding_rs` (v0.8.35) : Pour la gestion des encodages de fichiers.
- `encoding_rs_io` (v0.1.7) : Pour la lecture de fichiers avec différents encodages.
- `ignore` (v0.4.23) : Pour ignorer les fichiers et répertoires.
- `regex` (v1.11.1) : Pour la recherche avec des expressions régulières.

## Installation

1.  Assurez-vous d'avoir [Rust](https://www.rust-lang.org/tools/install) installé.
2.  Clonez ce dépôt :
    ```sh
    git clone [https://github.com/votre-utilisateur/finder.git](https://github.com/cederig/finder.git)
    cd finder
    ```
3.  Compilez le projet :
    ```sh
    cargo build --release
    ```
    L'exécutable se trouvera dans `target/release/finder`.

## Compilation pour Windows

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

## Ignorer des fichiers

`finder` respecte automatiquement les règles définies dans les fichiers `.gitignore` et `.ignore`. Cela signifie que les fichiers et répertoires qui sont généralement ignorés dans un projet (comme `target/`, `node_modules/`, etc.) seront automatiquement exclus de la recherche. Vous pouvez personnaliser ce comportement en créant vos propres fichiers `.ignore` dans votre projet.

```sh
finder [OPTIONS] <PATTERN> <PATHS>...
```

### Arguments

-   `<PATTERN>` : L'expression régulière (regex) à rechercher.
-   `<PATHS>...` : Un ou plusieurs chemins de fichiers ou de répertoires dans lesquels rechercher.

### Options

-   `-i`, `--ignore-case` : Effectue une recherche insensible à la casse.
-   `-s`, `--stat` : Affiche des statistiques détaillées après la recherche.
-   `-h`, `--help` : Affiche le message d'aide.
-   `-V`, `--version` : Affiche la version de l'outil.

### Exemples

-   Rechercher "hello" dans un fichier (le mot "hello" sera surligné en rouge) :
    ```sh
    finder "hello" mon_fichier.txt
    ```

-   Rechercher "error" dans plusieurs fichiers :
    ```sh
    finder "error" fichier1.log fichier2.log
    ```

-   Rechercher "TODO" dans un répertoire entier :
    ```sh
    finder "TODO" ./mon_projet/
    ```

-   Rechercher avec des statistiques :
    ```sh
    finder --stat "important" ./docs/
    ```

## Tests

Pour exécuter les tests unitaires, utilisez la commande suivante :

```sh
cargo test
```
