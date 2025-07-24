# Finder

`finder` is a simple and fast command-line tool written in Rust to search for a string in files and directories.

## Features

- Search for a single string or multiple strings from a file.
- Recursive search in directories.
- Display of the line number, the matching pattern, and the content of the matching line.
- Progress bar during the search.
- Search statistics (number of matches, elapsed time, etc.).
- Parallel processing for faster searches.
- Colored output for better readability.
- Handles both Windows (CRLF) and Unix (LF) line endings correctly.

## Dependencies

This project uses the following Rust dependencies (as defined in `Cargo.toml`):

- `clap` (version `4.5.41`) : For command-line argument parsing.
- `indicatif` (version `0.18.0`) : For displaying a progress bar.
- `rayon` (version `1.10.0`) : For parallel processing.
- `colored` (version `2.1.0`) : For coloring terminal output.
- `encoding_rs` (version `0.8.35`) : For file encoding management.
- `encoding_rs_io` (version `0.1.7`) : For reading files with different encodings.
- `ignore` (version `0.4.23`) : For ignoring files and directories.
- `regex` (version `1.11.1`) : For regular expression searching.
- `tempfile` (version `3.20.0`) : For creating temporary files and directories in tests.

## Installation

### Prerequisites

Make sure you have Rust and Cargo installed on your system. You can install them by following the instructions on the official Rust website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compiling for Linux (from Linux/macOS)
1.  Clone this repository:
    ```sh
    git clone https://github.com/cederig/finder.git
    cd finder
    ```
2.  Compile the project:
    ```sh
    cargo build --release
    ```
    The executable will be located in `target/release/finder`.

### Compiling for Windows (from Linux/macOS)

To cross-compile this project for Windows from another operating system (like Linux or macOS), you will need the Rust target for Windows.

1.  Add the Windows target to your Rust installation:
    ```sh
    rustup target add x86_64-pc-windows-gnu
    ```

2.  Compile the project for the Windows target:
    ```sh
    cargo build --release --target=x86_64-pc-windows-gnu
    ```

The Windows executable will be located in `target/x86_64-pc-windows-gnu/release/finder.exe`.

### Compiling for macOS (from Linux/macOS)

To cross-compile this project for macOS from another operating system (like Linux or macOS), you will need the Rust target for macOS.

1.  Add the macOS target to your Rust installation (choose the correct architecture):
    *   For Intel Macs (x86_64):
        ```sh
        rustup target add x86_64-apple-darwin
        ```
    *   For Apple Silicon Macs (aarch64):
        ```sh
        rustup target add aarch64-apple-darwin
        ```

2.  Compile the project for the macOS target (choose the correct architecture):
    *   For Intel Macs:
        ```sh
        cargo build --release --target=x86_64-apple-darwin
        ```
    *   For Apple Silicon Macs:
        ```sh
        cargo build --release --target=aarch64-apple-darwin
        ```

The macOS executable will be located in `target/<your_mac_target>/release/finder` (e.g., `target/x86_64-apple-darwin/release/finder`).

## Usage

```bash
finder [OPTIONS] <PATHS>... -p <PATTERN>
finder [OPTIONS] <PATHS>... -f <FILE>
```

### Arguments

-   `<PATHS>...` : One or more file or directory paths to search within.

### Options

-   `-p`, `--pattern <PATTERN>` : The string to search for. Mutually exclusive with `-f`.
-   `-f`, `--input-file <FILE>` : Search for patterns from a file (one per line). Mutually exclusive with `-p`.
-   `-i`, `--ignore-case` : Performs a case-insensitive search.
-   `-o`, `--output <FILE>` : Exports results to the specified file instead of displaying them on the console.
-   `-s`, `--stat` : Displays detailed statistics after the search.
-   `-h`, `--help` : Displays help message.
-   `-V`, `--version` : Displays the tool version.

## Examples

-   Search for "hello" in a file (the word "hello" will be highlighted in red):
    ```sh
    ./finder my_file.txt -p "hello"
    ```

-   Search for "error" in multiple files:
    ```sh
    ./finder file1.log file2.log -p "error"
    ```

-   Search for all patterns in `patterns.txt` in an entire directory:
    ```sh
    ./finder ./my_project/ -f patterns.txt
    ```

-   Search with statistics:
    ```sh
    ./finder ./docs/ --stat -p "important"
    ```

## Output Format

The output format is as follows:
`path/to/file:line_number:matching_pattern:line_content_with_highlight`

## Ignoring Files

`finder` automatically respects rules defined in `.gitignore` and `.ignore` files. This means that files and directories typically ignored in a project (like `target/`, `node_modules/`, etc.) will be automatically excluded from the search. You can customize this behavior by creating your own `.ignore` files in your project.


## Tests

This project includes unit tests; to run them, use the following command at the project root:

```sh

cargo test
```

This command compiles the program in test mode and executes all test functions.