use std::{path::Path, io::{Write, self}};
use yaml_rust::{YamlLoader, ScanError, Yaml};
use color_print::cprintln;
use std::env;
use no_panic::no_panic;



const HELP_MESSAGE: &str = "
cleanstore - a tool to clean up your the files that macOS won't
Usage: cleanstore (OPTION) 
Options:
    --help (-h or help) :  prints this message then exits
    --version (-v or version) : prints the version of cleanstore then exits
    --silent (-s) : runs cleanstore in silent mode, no output is printed

";
//     --daemon (-d) : runs cleanstore as a daemon process, requires root

const DEFAULT_CONFIG_FILE: &str = "
#
# cleanstore.yml
#

silent: false

# leaving blank causes default to home
root_directory: {}

to_remove: |
  {}/.lesshst
  {}/.wget-hsts

directories_to_ignore: |
  {}/Library
";

// [FILE]
//File:
//    Alternative root directory to clean up, defaults to the home directory


struct DiskSpace {
    unit: DataType,
    amount: f32,
}

impl DiskSpace {
    pub fn new() -> DiskSpace {
        DiskSpace {
            unit: DataType::B,
            amount: 0.0,
        }
    }

    pub fn add(&mut self, amount: u64, unit: DataType) { // taking only this causes overflow at 2GB
        
        // this approach is not ideal, but it works to hopefully minimize the amount of overflow

        let altered_ammount = match unit {
            DataType::B => amount,
            DataType::KB => amount * 1000,
            DataType::MB => amount * 1000 * 1000,
            DataType::GB => amount * 1000 * 1000 * 1000,
            DataType::TB => amount * 1000 * 1000 * 1000 * 1000,
        };
        
        self.amount += match self.unit {
            DataType::B => f64::from(altered_ammount as u32),
            DataType::KB => f64::from(altered_ammount as u32) / (1000.0),
            DataType::MB => f64::from(altered_ammount as u32) / (1000.0 * 1000.0),
            DataType::GB => f64::from(altered_ammount as u32) / (1000.0 * 1000.0 * 1000.0),
            DataType::TB => f64::from(altered_ammount as u32) / (1000.0 * 1000.0 * 1000.0 * 1000.0),
        } as f32;
        self.normalize();
    }

    fn to_string(&self) -> String {
        match self.unit {
            DataType::B => format!("{} B", self.amount),
            DataType::KB => format!("{} KB", self.amount),
            DataType::MB => format!("{} MB", self.amount),
            DataType::GB => format!("{} GB", self.amount),
            DataType::TB => format!("{} TB", self.amount),
        }
    }

    fn normalize(&mut self) {
        if self.amount > 1000.0 {
            self.amount /= 1000.0;
            self.unit = match self.unit {
                DataType::B => DataType::KB,
                DataType::KB => DataType::MB,
                DataType::MB => DataType::GB,
                DataType::GB => DataType::TB,
                DataType::TB => panic!("cleanstore: overflowed disk space"),
            };
        }
    }
}

enum DataType {
    B,
    KB,
    MB,
    GB,
    TB,
}

#[no_panic]
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
                println!("cleanstore v0.1.1");
                return;
            },
            //"--daemon" | "-d" => {
            //    println!("cleanstore: daemon mode is not yet implemented");
            //    return;
            //},
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

    let mut amount_removed: DiskSpace = DiskSpace::new();
    let home_dir = std::env::var("HOME").unwrap();
    //let _current_dir = env::current_dir().unwrap();

    // ------------------------------------------------------------------------------------------------------------

    let config_data_res = read_config_file();
    let config_data = match config_data_res {
        Ok(data) => data,
        Err(_) => panic!("cleanstore: could not read config file"),
    };

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
            //if !is_silent { cprintln!("<bold>***</bold> <red>File does not exist: {}</red>", file.display()); }
        }
    }

    // ------------------------------------------------------------------------------------------------------------

    if !is_silent { cprintln!("<green, bold>[+]</green, bold> <yellow>Removing .DS_Store files...</yellow>"); }

    // iterate through all the files in the home directory of the user and remove the files that are named exactly '.DS_Store'
    // when each of the files are removed there size should be added to the amount_removed variable
    // the amount_removed variable should be converted to the largest unit possible
    find_files_rec(root_directory, &mut amount_removed, &directories_to_ignore, &is_silent);

    // ------------------------------------------------------------------------------------------------------------

    cprintln!("<green>[+]</green> <yellow>Liberated a total of {} bytes!</yellow>", amount_removed.to_string());
}

fn find_files_rec(current_directory: &Path, amount_removed: &mut DiskSpace, directories_to_ignore: &Vec<&Path>, is_silent: &bool) {
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

fn remove_file(file_to_remove: &Path, amount_removed: &mut DiskSpace, is_silent: &bool) {
    // get the size of the file and add it to the amount_removed variable

    let file_size = file_to_remove.metadata().unwrap().len();
    amount_removed.add(file_size, DataType::B);

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

fn read_config_file() -> Result<String, std::io::Error> {

    let home_dir = std::env::var("HOME").unwrap();
    let data = std::fs::read_to_string(home_dir.clone() + "/.config/cleanstore/config.yml");

    match data {
        Ok(config_data) => {
            Ok(config_data)
        },
        Err(e) => {
            // write a new config file after propting the user
            print!("cleanstore: config file not found, would you like to create a new one? [Y/n] ");
            let _ = io::stdout().flush(); // need to ensure that the print statement is flushed to the terminal
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            if input.starts_with("y") || input.starts_with("Y") || input.eq("\n") {
                println!("cleanstore: creating new config file...");
                let config = DEFAULT_CONFIG_FILE.replace("{}", &home_dir);
                let mut config_file = std::fs::File::create(home_dir.clone() + "/.config/cleanstore/config.yml").unwrap();
                let _ = config_file.write_all(config.as_bytes());
                Ok(config.to_string())
            } else {
                println!("cleanstore: exiting...");
                Err(e)
            }
        },
    }
}
