use filetime::FileTime;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::process;
use std::time::SystemTime;

struct Options {
    no_create: bool, // Prevent file creation if it doesn't exist
    directory: bool, // Create directories instead of files
    acc_time: bool,  // Update only access time
    mod_time: bool,  // Update only modification time
}

impl Options {
    fn new() -> Self {
        Options {
            no_create: false,
            directory: false,
            acc_time: false,
            mod_time: false,
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect(); // Collect command-line arguments
    let path = env::current_exe()?; // Get the path of the currently running program
    let file_name_exe = path.file_name().unwrap().to_string_lossy(); // Extract the executable name
    let file_name = path.file_stem().unwrap().to_string_lossy(); // Extract the file name without extension

    let arguments = [
        "/h",
        "--help",
        "/c",
        "--no-create",
        "/d",
        "--directory",
        "/a",
        "--access-time",
        "/m",
        "--modification-time",
    ]; // Recognized arguments
    let mut options = Options::new(); // Initialize options

    check_args(&args, &file_name, &mut options); // Parse and set options based on args passed in

    for arg in args.iter() {
        if (arg == &file_name.to_string() || arg == &file_name_exe.to_string())
            || (arguments.contains(&arg.as_str()))
        // assume user does not want to name a file/folder one of the arguments
        {
            continue; // Skip the executable name and recognized arguments
        }

        make_file(arg, &options); // Process file creation or updates
    }

    Ok(())
}

// Parse command-line arguments and set options accordingly
fn check_args(args: &[String], file_name: &str, options: &mut Options) {
    if args.len() == 1 {
        eprintln!("Usage: {}[.exe] [args] <file_name>", file_name);
        eprintln!("enter '{} --help' to learn more", file_name);
        process::exit(1);
    }

    let arg = &args[1];

    if arg == "/h" || arg == "--help" {
        // Display help information
        println!("Usage: {}[.exe] [args] <file_name>", file_name);
        println!("\nOPTIONAL ARGUMENTS\n");
        println!("Note: Only one argument can be used at a time\n");
        println!("/h or --help                Displays this help message\n");
        println!("/c or --no-create           Prevents creating new files if they don't already exist\n                            If a file exists, the timestamps will be updated\n                            but if the file doesn't exist, nothing will happen\n");
        println!("\n/d or --directory           Creates directories instead of files\n                            Can also create nested folders:\n                            {} --directory this/is/a/nested/folder\n", file_name);
        println!("/a or --access-time         Only updates the accessed time of the file if the file already exists,\n                            otherwise it creates the file like normal\n");
        println!("\n/m or --modification-time   Only updates the modified time of the file if the file already exists,\n                            otherwise it creates the file like normal\n");

        println!("\nMade  by rip-super on Github (https://github.com/rip-super)");
        
        process::exit(0);
    } else if arg == "/c" || arg == "--no-create" {
        options.no_create = true;
    } else if arg == "/d" || arg == "--directory" {
        options.directory = true;
    } else if arg == "/a" || arg == "--access-time" {
        options.acc_time = true;
    } else if arg == "/m" || arg == "--modification-time" {
        options.mod_time = true;
    }
}

// Create or update a file or directory based on the provided options
fn make_file(file_name: &String, options: &Options) {
    if options.no_create && !std::path::Path::new(file_name).exists() {
        // Skip creating the file if --no-create is specified and it doesn't exist
    } else if options.directory {
        match fs::create_dir_all(file_name) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                eprintln!(
                    "Error: Unable to create the folder '{}'.\nPossible reasons:\n - Insufficient permissions.\n - Invalid folder name.",
                    file_name
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("An unexpected error occurred: {}", e);
                process::exit(1);
            }
        }
    } else if options.acc_time {
        create_file(file_name, "a"); // Update access time only
    } else if options.mod_time {
        create_file(file_name, "m"); // Update modification time only
    } else {
        create_file(file_name, "none"); // Create or update both timestamps
    }
}

// Handle file creation and timestamp updates
fn create_file(file_name: &String, type_: &str) {
    match File::create_new(file_name) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!(
                "Error: Unable to create the file '{}'.\nPossible reasons:\n - Insufficient permissions.\n - Invalid file name.",
                file_name
            );
            process::exit(1);
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            let now = SystemTime::now();
            let now_filetime = FileTime::from_system_time(now);

            if type_ == "a" {
                filetime::set_file_atime(file_name, now_filetime).unwrap(); // Update access time
            } else if type_ == "m" {
                filetime::set_file_mtime(file_name, now_filetime).unwrap(); // Update modification time
            } else {
                // Update both
                filetime::set_file_times(file_name, now_filetime, now_filetime).unwrap();
            }
        }
        Err(e) => {
            eprintln!("An unexpected error occurred: {}", e);
            process::exit(1);
        }
    }
}
