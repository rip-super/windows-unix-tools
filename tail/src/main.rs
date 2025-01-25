use std::env;
use std::fs;
use std::io::{self, Read, Seek};
use std::process;

struct Options {
    num_lines: u32,
    num_bytes: Option<u32>,
}

impl Options {
    fn new() -> Self {
        Options {
            num_lines: 10,
            num_bytes: None,
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

    display_files(&args, &file_name, &mut options);

    Ok(())
}

fn parse_size(size_str: &str) -> Option<u64> {
    let chars = size_str.chars();
    let value: String = chars.clone().take_while(|c| c.is_ascii_digit()).collect();
    let unit: String = chars.skip_while(|c| c.is_ascii_digit()).collect();

    let value: u64 = value.parse().ok()?;

    match unit.to_lowercase().as_str() {
        "k" => Some(value * 1024),               // Kilobytes
        "m" => Some(value * 1024 * 1024),        // Megabytes
        "g" => Some(value * 1024 * 1024 * 1024), // Gigabytes
        "" => Some(value),                       // Plain bytes
        _ => None,                               // Invalid unit
    }
}

fn check_args(args: &[String], file_name: &str, options: &mut Options) {
    if args.len() == 1 {
        eprintln!("Usage: {}[.exe] [args] <file_name>", file_name);
        eprintln!("Enter '{} --help' to learn more", file_name);
        process::exit(1);
    }

    let arg = &args[1];

    if arg == "/h" || arg == "--help" {
        println!("Usage: {}[.exe] [args] <file_name>", file_name);
        println!("\nOPTIONAL ARGUMENTS\n");
        println!("Note: Only one argument can be used at a time\n");
        println!("/h or --help                  Displays this help message");
        println!("/l or --num-lines <number>    Displays the first n lines of the file");
        println!("/b or --num-bytes <size>      Displays the first n bytes of the file");
        println!("                              (Also supports human-readable formats like:");
        println!("                              '2k' for 2 kilobytes,");
        println!("                              '3m' for 3 megabytes");
        println!("                              and '1g' for 1 gigabyte)");
        process::exit(0);
    } else if arg == "/l" || arg == "--num-lines" {
        let num_lines = args[2].parse::<u32>();
        match num_lines {
            Ok(s) => options.num_lines = s,
            Err(_) => {
                eprintln!("Error: Expected value after /l or --num-lines flag to be a positive whole number");
                eprintln!("Enter '{} --help' to learn more", file_name);
                process::exit(1);
            }
        }
    } else if arg == "/b" || arg == "--num-bytes" {
        if args.len() > 2 {
            match parse_size(&args[2]) {
                Some(size) => options.num_bytes = Some(size as u32),
                None => {
                    eprintln!(
                        "Error: Invalid size format '{}' after /b or --num-bytes flag",
                        args[2]
                    );
                    eprintln!("Enter '{} --help' to learn more", file_name);
                    process::exit(1);
                }
            }
        } else {
            eprintln!("Error: Missing value after /b or --num-bytes flag");
            eprintln!("Enter '{} --help' to learn more", file_name);
            process::exit(1);
        }
    }
}

fn display_files(args: &[String], file_name: &str, options: &mut Options) {
    let mut file_paths = vec![];
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with('-') || arg.starts_with('/') {
            match arg.as_str() {
                "--num-lines" | "/n" => {
                    if let Some(num_lines_str) = args.get(i + 1) {
                        if let Ok(num_lines) = num_lines_str.parse::<u32>() {
                            options.num_lines = num_lines;
                        } else {
                            eprintln!("Error: Invalid number of lines '{}'", num_lines_str);
                            eprintln!("Enter '{} --help' to learn more", file_name);
                            process::exit(1);
                        }
                    } else {
                        eprintln!("Error: Missing value for '{}'", arg);
                        eprintln!("Enter '{} --help' to learn more", file_name);
                        process::exit(1);
                    }
                    i += 2;
                    continue;
                }
                "--num-bytes" | "/b" => {
                    if let Some(num_bytes_str) = args.get(i + 1) {
                        match parse_size(num_bytes_str) {
                            Some(size) => options.num_bytes = Some(size as u32),
                            None => {
                                eprintln!("Error: Invalid size format '{}'", num_bytes_str);
                                eprintln!("Enter '{} --help' to learn more", file_name);
                                process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("Error: Missing value for '{}'", arg);
                        eprintln!("Enter '{} --help' to learn more", file_name);
                        process::exit(1);
                    }
                    i += 2;
                    continue;
                }
                _ => {
                    eprintln!("Error: Unknown argument '{}'", args[i]);
                    process::exit(1);
                }
            }
        } else if arg.as_str() == file_name {
            i += 1;
            continue;
        } else {
            file_paths.push(arg.clone());
            i += 1;
        }

        for file_path in &file_paths {
            match fs::File::open(file_path) {
                Ok(file) => {
                    let file_len = file.metadata().unwrap().len();
                    let mut reader = io::BufReader::new(file);

                    let string = format!("==> \x1b[32m{}\x1b[0m <==", file_path);
                    println!("{}", string);
                    println!("{}", "-".repeat(string.len()));

                    if let Some(num_bytes) = options.num_bytes {
                        let start_pos = if file_len > num_bytes as u64 {
                            file_len - num_bytes as u64
                        } else {
                            0
                        };

                        // Seek to the calculated position
                        reader.seek(io::SeekFrom::Start(start_pos)).unwrap();

                        // Create a buffer and read the bytes
                        let mut buffer = vec![0; num_bytes as usize];
                        let bytes_read = reader.read(&mut buffer).unwrap_or(0);

                        // Print the result
                        print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
                    } else {
                        let mut contents = String::new();
                        reader.read_to_string(&mut contents).unwrap();
                        let contents: Vec<&str> = contents.split('\n').collect();

                        // Display the **last** `options.num_lines` lines
                        let num_lines = options.num_lines as usize;
                        let start_index = if contents.len() > num_lines {
                            contents.len() - num_lines
                        } else {
                            0
                        };

                        for line in &contents[start_index..] {
                            println!("{}", line);
                        }
                    }

                    println!();
                }
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", &file_path, e);
                    process::exit(1);
                }
            }
        }
    }
}
