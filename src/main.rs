use clap::Parser;
use std::fs;
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use colored::*;
use std::collections::HashSet;
use encoding_rs::{Encoding, WINDOWS_1252};
use encoding_rs_io::DecodeReaderBytesBuilder;
use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The regex pattern to search for
    pattern: String,

    /// The path(s) to search in (files or directories)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Show statistics about the search
    #[arg(short, long)]
    stat: bool,

    /// Case-insensitive search
    #[arg(short, long)]
    ignore_case: bool,
}

struct SearchResult {
    path: PathBuf,
    line_number: usize,
    line: String,
}

fn search_in_file(path: &Path, re: &Regex) -> io::Result<Vec<SearchResult>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 4];
    let n = file.read(&mut buffer)?;
    file.seek(SeekFrom::Start(0))?;

    let (encoding, _) = Encoding::for_bom(&buffer[..n]).unwrap_or((WINDOWS_1252, 0));

    let transcoded_reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .build(file);

    let buf_reader = io::BufReader::new(transcoded_reader);
    let mut results = Vec::new();

    for (index, line) in buf_reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{} Error reading line from {}: {}", "warning:".yellow(), path.display(), e);
                continue;
            }
        };
        if re.is_match(&line) {
            results.push(SearchResult {
                path: path.to_path_buf(),
                line_number: index + 1,
                line,
            });
        }
    }
    Ok(results)
}

fn partition_paths(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    paths.into_iter().partition(|p| p.exists())
}

fn main() {
    let args = Args::parse();
    let start_time = Instant::now();

    let re = match RegexBuilder::new(&args.pattern)
        .case_insensitive(args.ignore_case)
        .build() {
        Ok(re) => re,
        Err(e) => {
            eprintln!("{} Invalid regex: {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let (valid_paths, invalid_paths) = partition_paths(args.paths);

    for path in &invalid_paths {
        eprintln!(
            "{}: {}: No such file or directory",
            "error".red().bold(),
            path.display()
        );
    }

    if valid_paths.is_empty() {
        eprintln!("{}", "No valid paths provided. Exiting.".yellow());
        if invalid_paths.is_empty() { return; }
        std::process::exit(1);
    }

    let mut walk_builder = WalkBuilder::new(&valid_paths[0]);
    if valid_paths.len() > 1 {
        for path in &valid_paths[1..] {
            walk_builder.add(path);
        }
    }

    let files_to_search: Vec<PathBuf> = walk_builder.build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .map(|e| e.into_path())
        .collect();

    if files_to_search.is_empty() {
        println!("No files to search in the provided paths.");
        return;
    }

    let pb = ProgressBar::new(files_to_search.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)").unwrap()
        .progress_chars("#>-"));

    let results: Vec<SearchResult> = files_to_search
        .par_iter()
        .filter_map(|path| {
            pb.inc(1);
            match search_in_file(path, &re) {
                Ok(search_results) => Some(search_results),
                Err(e) => {
                    eprintln!("{} Failed to read file {}: {}", "error:".red().bold(), path.display(), e);
                    None
                }
            }
        })
        .flatten()
        .collect();

    pb.finish_with_message("Search complete");

    for result in &results {
        let highlighted_line = re.replace_all(&result.line, |caps: &regex::Captures| {
            caps[0].red().bold().to_string()
        });
        println!(
            "{}:{}:{}",
            result.path.display().to_string().green(),
            result.line_number.to_string().yellow(),
            highlighted_line.trim()
        );
    }

    if args.stat {
        let elapsed = start_time.elapsed();
        let total_matches = results.len();
        let files_with_matches: HashSet<_> = results.iter().map(|r| r.path.clone()).collect();

        println!("\n--- Statistics ---");
        println!("Total matches found: {}", total_matches);
        println!("Files with matches: {}", files_with_matches.len());
        println!("Time elapsed: {:?}", elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use encoding_rs::WINDOWS_1252;

    fn create_test_file(path: &str, content: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_search_in_file_found() {
        let test_file = "test_found.txt";
        create_test_file(test_file, "hello world\nfind me here\nanother line");
        let re = Regex::new("find me").unwrap();
        let results = search_in_file(Path::new(test_file), &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 2);
        assert_eq!(results[0].line, "find me here");
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_search_in_file_not_found() {
        let test_file = "test_not_found.txt";
        create_test_file(test_file, "hello world\nanother line");
        let re = Regex::new("missing").unwrap();
        let results = search_in_file(Path::new(test_file), &re).unwrap();
        assert!(results.is_empty());
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_search_in_file_multiple_matches() {
        let test_file = "test_multiple.txt";
        create_test_file(test_file, "match one\nsome line\nmatch two");
        let re = Regex::new("match").unwrap();
        let results = search_in_file(Path::new(test_file), &re).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_partition_paths() {
        let valid_file = "valid_path.txt";
        create_test_file(valid_file, "content");
        let invalid_path = "non_existent_file.txt";

        let paths = vec![PathBuf::from(valid_file), PathBuf::from(invalid_path)];
        let (valid, invalid) = partition_paths(paths);

        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], PathBuf::from(valid_file));
        assert_eq!(invalid.len(), 1);
        assert_eq!(invalid[0], PathBuf::from(invalid_path));

        fs::remove_file(valid_file).unwrap();
    }

    #[test]
    fn test_search_in_file_windows1252_encoding() {
        let test_file = "test_windows1252.txt";
        let (encoded_content, _, _) = WINDOWS_1252.encode("Héllö Wörld");
        let mut file = fs::File::create(test_file).unwrap();
        file.write_all(&encoded_content).unwrap();

        let re = Regex::new("Héllö").unwrap();
        let results = search_in_file(Path::new(test_file), &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, "Héllö Wörld");

        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_search_in_file_case_insensitive() {
        let test_file = "test_case_insensitive.txt";
        create_test_file(test_file, "Hello hello HeLLo");
        let re = RegexBuilder::new("hello").case_insensitive(true).build().unwrap();
        let results = search_in_file(Path::new(test_file), &re).unwrap();
        assert_eq!(results.len(), 1);
        fs::remove_file(test_file).unwrap();
    }
}