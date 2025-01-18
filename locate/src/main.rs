use regex::Regex;
use std::env;
use std::io;
use std::process;
use walkdir::WalkDir;

struct Options {
    base_name: bool,
    case_sens: bool,
    count: bool,
    limit: u32,
    regex: Option<Regex>,
}

impl Options {
    fn new() -> Self {
        Options {
            base_name: false,
            case_sens: false,
            count: false,
            limit: u32::MAX,
            regex: None,
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::new();
    let file_name = env::current_exe()?
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    check_args(&args, &file_name, &mut options);

    let search_term = &args[args.len() - 1];
    find_file(search_term, &options);

    Ok(())
}

fn check_args(args: &[String], file_name: &str, options: &mut Options) {
    if args.len() == 1 || ((args[1] == "/l" || args[1] == "--limit") && args.len() > 4) {
        eprintln!("Usage: {}[.exe] [args] <search_term>", file_name);
        eprintln!("enter '{} --help' to learn more", file_name);
        process::exit(1);
    }

    let arg = &args[1];

    if arg == "/h" || arg == "--help" {
        println!("Usage: {}[.exe] [args] <search_term>", file_name);
        println!("\nOPTIONAL ARGUMENTS\n");
        println!("Note: Only one argument can be used at a time\n");
        println!("/h or --help              Displays this help message");
        println!("/b or --basename          Searches for files using their basename instead of their full path (case-insenitive)");
        println!("/s or --case-sensitive    Searches for files using case-sensitive search");
        println!("/c or --count             Only displays the number of matches and not the files matched");
        println!("/l or --limit <number>    Limits the number of results displayed");
        println!("/r or --regex <regexp>    Searches for files based on a regular expression");

        println!("\nMade  by rip-super on Github (https://github.com/rip-super)");

        process::exit(0)
    } else if arg == "/b" || arg == "--basename" {
        options.base_name = true;
    } else if arg == "/s" || arg == "--case-sensitive" {
        options.case_sens = true;
    } else if arg == "/c" || arg == "--count" {
        options.count = true;
    } else if arg == "/l" || arg == "--limit" {
        let limit = args[2].parse::<u32>();
        match limit {
            Ok(s) => options.limit = s,
            Err(_) => {
                eprintln!("Error: Expected value after limit flag to be a positive whole number");
                eprintln!("enter '{} --help' to learn more", file_name);
                process::exit(1)
            }
        }
    } else if arg == "/r" || arg == "--regex" {
        let regex = Regex::new(&args[2]);
        match regex {
            Ok(s) => options.regex = Some(s),
            Err(_) => {
                eprintln!(
                    "Error: Expected expression after regex flag to be a valid regular expression"
                );
                eprintln!("enter '{} --help' to learn more", file_name);
                process::exit(1)
            }
        }
    }
}

fn find_file(search_term: &str, options: &Options) {
    let path = ".";
    let mut files = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(e) => e,         // Successfully retrieve entry
            Err(_) => continue, // Skip entries that cannot be accessed
        };

        let metadata = match entry.metadata() {
            Ok(meta) => meta,   // Successfully retrieve metadata
            Err(_) => continue, // Skip files with inaccessible metadata
        };

        if metadata.is_file() && matches_search(entry.path(), search_term, options) {
            files.push(entry.path().display().to_string());
        }
    }

    if !options.count {
        for (idx, file) in files.clone().into_iter().enumerate() {
            if idx as u32 == options.limit {
                break;
            }

            // Highlight the matched part
            let highlighted = file.replace(
                search_term,
                &format!("\x1b[32m{}\x1b[0m", search_term), // Green ANSI escape codes
            );
            println!("{}", highlighted);
        }
    }

    println!("\n{} results found", files.len());
}

fn matches_search(path: &std::path::Path, search_term: &str, options: &Options) -> bool {
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        if options.base_name {
            // Case-insensitive search in base name
            file_name
                .to_lowercase()
                .contains(&search_term.to_lowercase())
        } else if options.case_sens {
            // Case-sensitive search in the full path
            path.display().to_string().contains(search_term)
        } else if options.regex.is_some() {
            // search with regex
            options
                .regex
                .clone()
                .unwrap()
                .is_match(&path.display().to_string().to_lowercase())
        } else {
            // Case-insensitive search in full path
            path.display()
                .to_string()
                .to_lowercase()
                .contains(&search_term.to_lowercase())
        }
    } else {
        false
    }
}
