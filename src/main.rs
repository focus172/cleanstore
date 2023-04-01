extern crate color_print;

use color_print::cprintln;
use std::path::{Path, PathBuf};

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

    pub fn add(&mut self, amount: u64, unit: DataType) {
        // taking only this causes overflow at 2GB

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

fn main() {
    // ------------------------------------------------------------------------------------------------------------

    let args = std::env::args();

    let mut amount_removed: DiskSpace = DiskSpace::new();

    let arg: Option<String> = args.skip(1).next();
    let dir_name = match arg {
        Some(dir) => {
            if dir.starts_with("/") {
                dir
            } else if dir.starts_with("~") {
                let home_dir = std::env::var("HOME").unwrap();
                let mut home: PathBuf = PathBuf::from(home_dir);
                home.push(dir.chars().skip(2).collect::<String>());
                home.as_path().to_str().unwrap().to_string()
            } else {
                let mut cur_dir = PathBuf::from(std::env::current_dir().unwrap());
                cur_dir.push(dir);
                cur_dir.as_path().to_str().unwrap().to_string()
            }
        }
        None => std::env::var("HOME").unwrap(),
    };

    let root_directory = Path::new(&dir_name);

    // ------------------------------------------------------------------------------------------------------------

    cprintln!("<green, bold>[+]</green, bold> <yellow>Removing .DS_Store files...</yellow>");

    // iterate through all the files in the home directory of the user and remove the files that are named exactly '.DS_Store'
    // when each of the files are removed there size should be added to the amount_removed variable
    // the amount_removed variable should be converted to the largest unit possible
    find_files_rec(root_directory, &mut amount_removed);

    // ------------------------------------------------------------------------------------------------------------

    cprintln!(
        "<green>[+]</green> <yellow>Liberated a total of {} bytes!</yellow>",
        amount_removed.to_string()
    );
}

fn find_files_rec(current_directory: &Path, amount_removed: &mut DiskSpace) {
    let cur_dir = std::fs::read_dir(current_directory);

    let dir = match cur_dir {
        Ok(d) => d,
        Err(_) => return,
    };

    for directory_item in dir {
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

fn remove_file(file_to_remove: &Path, amount_removed: &mut DiskSpace) {
    // get the size of the file and add it to the amount_removed variable

    let file_size = file_to_remove.metadata().unwrap().len();
    amount_removed.add(file_size, DataType::B);

    cprintln!(
        "<blue>(-)</blue> Removing file: {} <yellow>({} bytes)</yellow>",
        file_to_remove.display(),
        file_size
    );

    std::fs::remove_file(file_to_remove).unwrap();
}
