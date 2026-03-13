use std::env;
use std::fs;
use std::io::{self, Read};
use rustblog_wip_for_summarize_in_zh_cn::Summarizer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: summarizer <file_path> [max_sentences]");
        eprintln!("   or: summarizer --stdin [max_sentences]");
        std::process::exit(1);
    }

    let max_sentences = if args.len() >= 3 {
        match args[2].parse::<usize>() {
            Ok(n) => n.max(1),
            Err(_) => {
                eprintln!("Error: max_sentences must be a positive integer");
                std::process::exit(1);
            }
        }
    } else {
        5
    };

    let text = if args[1] == "--stdin" {
        let mut input = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
        input
    } else {
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", args[1], e);
                std::process::exit(1);
            }
        }
    };

    let summarizer = Summarizer::new(max_sentences);
    let summary = summarizer.summarize(&text);

    if summary.is_empty() {
        println!("Warning: Unable to generate summary. The input text may be too short or empty.");
    } else {
        println!("Summary ({} sentences):\n", max_sentences);
        println!("{}", summary);
    }
}