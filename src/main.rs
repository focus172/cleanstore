use std::path::Path;
use yaml_rust::YamlLoader;

fn main() {
    let mut amount_removed = 0;
    
    let home_dir = std::env::var("HOME").unwrap();

    // ------------------------------------------------------------------------------------------------------------

    let config_data = std::fs::read_to_string(home_dir.clone() + "/.config/cleanstore/config.yml").unwrap();
    let data = YamlLoader::load_from_str(&config_data).unwrap();
    let document = &data[0];

    let is_silent_entry = document["silent"].as_bool();
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
        None => vec![Path::new("Library")],
    };


    if !is_silent {
        println!("[+] Read config file!");
    }

    //println!("root_directory: {}", root_directory);
    println!("to_remove: {:?}", to_remove);
    //println!("directories_to_ignore: {:?}", directories_to_ignore);
    //println!("is_silent: {}", is_silent);

    // ------------------------------------------------------------------------------------------------------------

    println!("[+] Removing config files...");
    // iterate through the files specified in the config file and remove them

    for file in to_remove {
        remove_file(file, &mut amount_removed)
    }

    println!("done!");

    // ------------------------------------------------------------------------------------------------------------

    println!("[+] Removing .DS_Store files...");

    // iterate through all the files in the home directory of the user and remove the files that are named exactly '.DS_Store'
    // when each of the files are removed there size should be added to the amount_removed variable
    // the amount_removed variable should be converted to the largest unit possible
    let home_dir_path = Path::new(&home_dir);

    find_files_rec(root_directory, &mut amount_removed);

    println!("done!");

     // ------------------------------------------------------------------------------------------------------------
}

fn find_files_rec(current_directory: &Path, amount_removed: &mut u64) {
    for directory_item in std::fs::read_dir(current_directory).unwrap() {
        let item = directory_item.unwrap();
        let path_to_item = item.path();

        if path_to_item.is_dir() {
            find_files_rec(&path_to_item, amount_removed);
        } else {
            if path_to_item.file_name().unwrap() == ".DS_Store" {
                remove_file(&path_to_item, amount_removed);
            }
        }
    }
}

fn remove_file(file_to_remove: &Path, amount_removed: &mut u64) {
  // get the size of the file and add it to the amount_removed variable

    let file_size = file_to_remove.metadata().unwrap().len();
    *amount_removed += file_size;

    println!(">(-) Removing file: {} ({} bytes)", file_to_remove.display(), file_size);
    
    //std::fs::remove_file(file_to_remove).unwrap();
}
