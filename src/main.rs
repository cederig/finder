use clap::{Parser, ArgGroup};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use colored::*;
use encoding_rs::{Encoding, WINDOWS_1252};

use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,
    override_usage = "finder [OPTIONS] <PATHS>... -p <PATTERN>\n       finder [OPTIONS] <PATHS>... -f <FILE>")]
#[command(group(
    ArgGroup::new("pattern_source")
        .required(true)
        .args(["pattern", "input_file"]),
))]
struct Args {
    /// The string to search for (mutually exclusive with -f)
    #[arg(short = 'p', long)]
    pattern: Option<String>,

    /// A file containing patterns to search for, one per line (mutually exclusive with -p)
    #[arg(short = 'f', long = "input-file")]
    input_file: Option<PathBuf>,

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

#[derive(Debug)]
struct SearchResult {
    path: PathBuf,
    line_number: usize,
    line: String,
    pattern: String,
}

fn search_in_file_streaming(path: &Path, regexes: &[Regex]) -> io::Result<Vec<SearchResult>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Optimized encoding detection - only read first 4KB for BOM detection
    let bom_sample_size = std::cmp::min(4096, buffer.len());
    let (encoding, bom_len) = Encoding::for_bom(&buffer[..bom_sample_size]).unwrap_or((WINDOWS_1252, 0));
    let (decoded_content, _, _) = encoding.decode(&buffer[bom_len..]);

    let mut results = Vec::new();
    for (index, line) in decoded_content.lines().enumerate() {
        for re in regexes {
            if re.is_match(line) {
                results.push(SearchResult {
                    path: path.to_path_buf(),
                    line_number: index + 1,
                    line: line.to_string(),
                    pattern: re.as_str().to_string(),
                });
                break;
            }
        }
    }
    Ok(results)
}

fn compile_regex_with_cache(patterns: &[String], ignore_case: bool) -> Result<Vec<Regex>, regex::Error> {
    let mut cache: HashMap<(String, bool), Regex> = HashMap::new();
    patterns.iter().map(|p| {
        let cache_key = (p.clone(), ignore_case);
        if let Some(cached_regex) = cache.get(&cache_key) {
            Ok(cached_regex.clone())
        } else {
            let regex = RegexBuilder::new(p)
                .case_insensitive(ignore_case)
                .build()?;
            cache.insert(cache_key, regex.clone());
            Ok(regex)
        }
    }).collect()
}

fn partition_paths(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    paths.into_iter().partition(|p| p.exists())
}

fn read_lines_from_file(path: &Path) -> io::Result<Vec<String>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Optimized encoding detection - only read first 4KB for BOM detection
    let bom_sample_size = std::cmp::min(4096, buffer.len());
    let (encoding, bom_len) = Encoding::for_bom(&buffer[..bom_sample_size]).unwrap_or((WINDOWS_1252, 0));
    let (decoded_content, _, _) = encoding.decode(&buffer[bom_len..]);

    Ok(decoded_content.lines().map(String::from).collect())
}

fn load_patterns(args: &Args) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if let Some(pattern) = &args.pattern {
        Ok(vec![pattern.clone()])
    } else if let Some(file_path) = &args.input_file {
        Ok(read_lines_from_file(file_path)?)
    } else {
        unreachable!("Either a pattern or an input file must be provided.");
    }
}

fn run_app(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let patterns = load_patterns(&args)?;
    let regexes = compile_regex_with_cache(&patterns, args.ignore_case)?;

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
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .map(|e| e.into_path())
        .collect();

    if files_to_search.is_empty() {
        println!("No files to search in the provided paths.");
        return Ok(());
    }

    let pb = Arc::new(Mutex::new(ProgressBar::new(files_to_search.len() as u64)));
    {
        let pb_guard = pb.lock().unwrap();
        pb_guard.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)").unwrap()
            .progress_chars("#>-"));
    }

    let regexes = Arc::new(regexes);
    let output_results = Arc::new(Mutex::new(Vec::new()));

    files_to_search.par_iter().for_each(|path| {
        {
            let pb_guard = pb.lock().unwrap();
            pb_guard.inc(1);
        }
        
        match search_in_file_streaming(path, &regexes) {
            Ok(search_results) => {
                if !search_results.is_empty() {
                    let mut output_guard = output_results.lock().unwrap();
                    output_guard.extend(search_results);
                }
            },
            Err(e) => {
                eprintln!("{} Failed to read file {}: {}", "error:".red().bold(), path.display(), e);
            }
        }
    });

    {
        let pb_guard = pb.lock().unwrap();
        pb_guard.finish_with_message("Search complete");
    }

    let results = Arc::try_unwrap(output_results).unwrap().into_inner().unwrap();

    if let Some(output_path) = &args.output {
        let mut output_file = fs::File::create(output_path)?;
        for result in &results {
            // In file output, we don't colorize, just output the raw data.
            writeln!(
                output_file,
                "{}:{}:{}:{}",
                result.path.display(),
                result.line_number,
                result.pattern,
                result.line.trim()
            )?;
        }
    } else {
        for result in &results {
            let re = RegexBuilder::new(&result.pattern)
                .case_insensitive(args.ignore_case)
                .build()?;
            let highlighted_line = re.replace_all(&result.line, |caps: &regex::Captures| {
                caps[0].red().bold().to_string()
            });
            println!(
                "{}:{}:{}:{}",
                result.path.display().to_string().green(),
                result.line_number.to_string().yellow(),
                result.pattern.magenta(),
                highlighted_line.trim()
            );
        }
    }

    if args.stat {
        let elapsed = start_time.elapsed();
        let total_matches = results.len();
        let files_with_matches: HashSet<_> = results.iter().map(|r| r.path.clone()).collect();

        println!("
--- Statistics ---");
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
        let re = vec![Regex::new("find me").unwrap()];
        let results = search_in_file_streaming(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 2);
        assert_eq!(results[0].line, "find me here");
        assert_eq!(results[0].pattern, "find me");
        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_not_found() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_not_found.txt");
        create_test_file(&test_file_path, "hello world\nanother line");
        let re = vec![Regex::new("missing").unwrap()];
        let results = search_in_file_streaming(&test_file_path, &re).unwrap();
        assert!(results.is_empty());
        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_multiple_matches() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_multiple.txt");
        create_test_file(&test_file_path, "match one\nsome line\nmatch two");
        let re = vec![Regex::new("match").unwrap()];
        let results = search_in_file_streaming(&test_file_path, &re).unwrap();
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

        let re = vec![Regex::new("Héllö").unwrap()];
        let results = search_in_file_streaming(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, "Héllö Wörld");

        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_case_insensitive() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_case_insensitive.txt");
        create_test_file(&test_file_path, "Hello hello HeLLo");
        let re = vec![RegexBuilder::new("hello").case_insensitive(true).build().unwrap()];
        let results = search_in_file_streaming(&test_file_path, &re).unwrap();
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
            pattern: Some("pattern".to_string()),
            input_file: None,
            paths: vec![input_file_path.clone()],
            stat: false,
            ignore_case: false,
            output: Some(output_file_path.clone()),
        };

        run_app(args).unwrap();

        let output_content = fs::read_to_string(&output_file_path).unwrap();
        let expected_content = format!(
            "{}:{}:pattern:Line 1 with pattern\n{}:{}:pattern:Another line with pattern\n",
            input_file_path.display(), 1,
            input_file_path.display(), 3
        );
        assert_eq!(output_content, expected_content);

        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_with_input_file() {
        let test_dir = tempdir().unwrap();
        let target_file_path = test_dir.path().join("target.txt");
        let patterns_file_path = test_dir.path().join("patterns.txt");

        create_test_file(&target_file_path, "This is line one.\nHere is the second line.\nAnd a third.");
        create_test_file(&patterns_file_path, "one\nthird");

        let args = Args {
            pattern: None,
            input_file: Some(patterns_file_path),
            paths: vec![target_file_path.clone()],
            stat: false,
            ignore_case: false,
            output: None,
        };

        // We can't directly test run_app and capture stdout easily without a more complex setup.
        // So we'll test the core logic parts.
        let patterns = load_patterns(&args).unwrap();
        assert_eq!(patterns, vec!["one", "third"]);

        let regexes: Vec<Regex> = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();
        let results = search_in_file_streaming(&target_file_path, &regexes).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[0].pattern, "one");
        assert_eq!(results[0].line, "This is line one.");

        assert_eq!(results[1].line_number, 3);
        assert_eq!(results[1].pattern, "third");
        assert_eq!(results[1].line, "And a third.");

        test_dir.close().unwrap();
    }

    #[test]
    fn test_search_in_file_with_crlf() {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("test_crlf.txt");
        create_test_file(&test_file_path, "line one\r\nline two\r\nline three");
        let re = vec![Regex::new("two").unwrap()];
        let results = search_in_file_streaming(&test_file_path, &re).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 2);
        assert_eq!(results[0].line, "line two");
        test_dir.close().unwrap();
    }
}
