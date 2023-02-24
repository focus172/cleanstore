use std::path::Path;
use yaml_rust::{YamlLoader, ScanError, Yaml};
use color_print::cprintln;
use std::env;

const HELP_MESSAGE: &str = "
cleanstore - a tool to clean up your the files that macOS won't
Usage: cleanstore [OPTION]
Options:
    --help
    --version
";

fn main() {

    // ------------------------------------------------------------------------------------------------------------
    
    let mut affirmative_silent = false;

    // parse the arguments
    for argument in env::args() {
        match argument.as_str() {
            "--help" | "-h" | "help" => {
                println!("{HELP_MESSAGE}");
                return;
            },
            "--version" | "-v" | "version" => {
                println!("cleanstore v0.1.0");
                return;
            },
            "--silent" | "-s" => {
                affirmative_silent = true;
            },
            other => {
                if !other.contains("cleanstore") {
                    println!("cleanstore: invalid option '{}'", argument);
                }
            },
        }
    }

    // ------------------------------------------------------------------------------------------------------------

    let mut amount_removed = 0;
    let home_dir = std::env::var("HOME").unwrap();
    let _current_dir = env::current_dir().unwrap();

    // ------------------------------------------------------------------------------------------------------------

    let config_data: String = std::fs::read_to_string(home_dir.clone() + "/.config/cleanstore/config.yml").unwrap();
    let wrapped_data: Result<Vec<Yaml>, ScanError> = YamlLoader::load_from_str(&config_data);
    let data: Vec<Yaml> = wrapped_data.unwrap(); // TODO: handle better
    let document = &data[0];

    let is_silent_entry = if affirmative_silent { Some(true) } else { document["silent"].as_bool() };
    let root_directory_entry = document["root_directory"].as_str();
    let to_remove_entry = document["to_remove"].as_str(); // should cast to lines and then to Path
    let directories_to_ignore_entry = document["directories_to_ignore"].as_str(); // should cast to lines and then to Path

    let is_silent = match is_silent_entry {
        Some(value) => value,
        None => false,
    };

    let root_directory = match root_directory_entry {
        Some(value) => Path::new(value),
        None => Path::new(&home_dir),
    };

    let to_remove: Vec<&Path> = match to_remove_entry {
        Some(list) => {
            list.lines()
                .map(|line| Path::new(line))
                .collect()
        },
        None => vec![Path::new(".lesshst"), Path::new(".wget-hsts")],
    };

    let directories_to_ignore: Vec<&Path> = match directories_to_ignore_entry {
        Some(list) => {
            list.lines()
                .map(|line| Path::new(line))
                .collect()
        },
        None => vec![Path::new("~/Library")],
    };


    if !is_silent { cprintln!("<green, bold>[+]</green, bold> <yellow>Parsed config file!</yellow>"); }

    // ------------------------------------------------------------------------------------------------------------

    if !is_silent { cprintln!("<green, bold>[+]</green, bold> <yellow>Removing files from config...</yellow>"); }

    // iterate through the files specified in the config file and remove them
    for file in to_remove {
        if file.exists() {
            remove_file(file, &mut amount_removed, &is_silent);
        } else {
            if !is_silent { cprintln!("<bold>***</bold> <red>File does not exist: {}</red>", file.display()); }
        }
    }

    // ------------------------------------------------------------------------------------------------------------

    if !is_silent { cprintln!("<green, bold>[+]</green, bold> <yellow>Removing .DS_Store files...</yellow>"); }

    // iterate through all the files in the home directory of the user and remove the files that are named exactly '.DS_Store'
    // when each of the files are removed there size should be added to the amount_removed variable
    // the amount_removed variable should be converted to the largest unit possible
    find_files_rec(root_directory, &mut amount_removed, &directories_to_ignore, &is_silent);

    // ------------------------------------------------------------------------------------------------------------

    cprintln!("<green>[+]</green> <yellow>Liberated a total of {} bytes!</yellow>", amount_removed);
}

fn find_files_rec(current_directory: &Path, amount_removed: &mut u64, directories_to_ignore: &Vec<&Path>, is_silent: &bool) {
    for directory_item in std::fs::read_dir(current_directory).unwrap() {
        let item = directory_item.unwrap();
        let path_to_item = item.path();

        if path_to_item.is_dir() && !should_ignore_file(directories_to_ignore, &path_to_item) {
            find_files_rec(&path_to_item, amount_removed, directories_to_ignore, is_silent);
        } else {
            if path_to_item.file_name().unwrap() == ".DS_Store" {
                remove_file(&path_to_item, amount_removed, is_silent);
            }
        }
    }
}

fn remove_file(file_to_remove: &Path, amount_removed: &mut u64, is_silent: &bool) {
    // get the size of the file and add it to the amount_removed variable

    let file_size = file_to_remove.metadata().unwrap().len();
    *amount_removed += file_size;

    if !is_silent { cprintln!("<blue>(-)</blue> Removing file: {} <yellow>({} bytes)</yellow>", file_to_remove.display(), file_size); }
    
    std::fs::remove_file(file_to_remove).unwrap();
}

fn should_ignore_file(directories_to_ignore: &Vec<&Path>, file_to_remove: &Path) -> bool {
    for directory in directories_to_ignore {
        if file_to_remove.ends_with(directory) {
            return true;
        }
    }

    return false;
}
