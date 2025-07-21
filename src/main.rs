use clap::Parser;
use std::fs;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
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

    /// Output results to a file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
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

fn run_app(args: Args) -> Result<(), Box<dyn std::error::Error>> {
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
        if invalid_paths.is_empty() { return Ok(()); }
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
        return Ok(());
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

    if let Some(output_path) = args.output {
        let mut output_file = fs::File::create(&output_path)?;
        for result in &results {
            let highlighted_line = re.replace_all(&result.line, |caps: &regex::Captures| {
                caps[0].to_string()
            });
            writeln!(
                output_file,
                "{}:{}:{}",
                result.path.display(),
                result.line_number,
                highlighted_line.trim()
            )?;
        }
    } else {
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

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run_app(args) {
        eprintln!("{} Application error: {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use encoding_rs::WINDOWS_1252;
    use tempfile::tempdir;

    fn create_test_file(path: &Path, content: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_search_in_file_found() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_found.txt");
        create_test_file(&test_file_path, "hello world\nfind me here\nanother line");
        let re = Regex::new("find me").unwrap();
        let results = search_in_file(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 2);
        assert_eq!(results[0].line, "find me here");
        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_not_found() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_not_found.txt");
        create_test_file(&test_file_path, "hello world\nanother line");
        let re = Regex::new("missing").unwrap();
        let results = search_in_file(&test_file_path, &re).unwrap();
        assert!(results.is_empty());
        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_multiple_matches() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_multiple.txt");
        create_test_file(&test_file_path, "match one\nsome line\nmatch two");
        let re = Regex::new("match").unwrap();
        let results = search_in_file(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
        test_dir.close().unwrap();
    }

    #[test]
    fn test_partition_paths() {
        let test_dir = tempdir().unwrap();
        let valid_file_path = test_dir.path().join("valid_path.txt");
        create_test_file(&valid_file_path, "content");
        let invalid_path = test_dir.path().join("non_existent_file.txt");

        let paths = vec![valid_file_path.clone(), invalid_path.clone()];
        let (valid, invalid) = partition_paths(paths);

        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], valid_file_path);
        assert_eq!(invalid.len(), 1);
        assert_eq!(invalid[0], invalid_path);

        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_windows1252_encoding() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_windows1252.txt");
        let (encoded_content, _, _) = WINDOWS_1252.encode("Héllö Wörld");
        let mut file = fs::File::create(&test_file_path).unwrap();
        file.write_all(&encoded_content).unwrap();

        let re = Regex::new("Héllö").unwrap();
        let results = search_in_file(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, "Héllö Wörld");

        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_case_insensitive() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_case_insensitive.txt");
        create_test_file(&test_file_path, "Hello hello HeLLo");
        let re = RegexBuilder::new("hello").case_insensitive(true).build().unwrap();
        let results = search_in_file(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 1);
        test_dir.close().unwrap();
    }

    #[test]
    fn test_output_to_file() {
        let test_dir = tempdir().unwrap();
        let input_file_path = test_dir.path().join("input.txt");
        let output_file_path = test_dir.path().join("output.txt");

        create_test_file(&input_file_path, "Line 1 with pattern\nLine 2\nAnother line with pattern");

        let args = Args {
            pattern: "pattern".to_string(),
            paths: vec![input_file_path.clone()],
            stat: false,
            ignore_case: false,
            output: Some(output_file_path.clone()),
        };

        run_app(args).unwrap();

        let output_content = fs::read_to_string(&output_file_path).unwrap();
        let expected_content = format!(
            "{}:{}:Line 1 with pattern\n{}:{}:Another line with pattern\n",
            input_file_path.display(), 1,
            input_file_path.display(), 3
        );
        assert_eq!(output_content, expected_content);

        test_dir.close().unwrap();
    }
}
