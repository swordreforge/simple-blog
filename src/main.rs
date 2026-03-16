use std::env;
use std::fs;
use std::io::{self, Read};
use rustblog_wip_for_summarize_in_zh_cn::{summarize_blog_post, Summarizer};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  summarizer <file_path> [max_sentences]           - Generate summary with N sentences");
        eprintln!("  summarizer --auto <file_path>                    - Auto-adjust summary length");
        eprintln!("  summarizer --blog <file_path>                    - Optimized for blog posts");
        eprintln!("  summarizer --stdin [max_sentences]               - Read from stdin");
        std::process::exit(1);
    }

    let mode = if args[1] == "--auto" {
        "auto"
    } else if args[1] == "--blog" {
        "blog"
    } else if args[1] == "--stdin" {
        "stdin"
    } else {
        "normal"
    };

    let text = if mode == "stdin" {
        let mut input = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
        input
    } else {
        let file_idx = if mode == "normal" { 1 } else { 2 };
        if args.len() <= file_idx {
            eprintln!("Error: file path required");
            std::process::exit(1);
        }
        match fs::read_to_string(&args[file_idx]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", args[file_idx], e);
                std::process::exit(1);
            }
        }
    };

    let summary = match mode {
        "blog" => summarize_blog_post(&text),
        "auto" => {
            let max_sentences = if args.len() >= 3 {
                args[2].parse::<usize>().unwrap_or(5)
            } else {
                5
            };
            Summarizer::new(max_sentences).summarize_auto(&text)
        }
        _ => {
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
            Summarizer::new(max_sentences).summarize(&text)
        }
    };

    if summary.is_empty() {
        println!("Warning: Unable to generate summary. The input text may be too short or empty.");
    } else {
        println!("Summary:\n{}", summary);
    }
}