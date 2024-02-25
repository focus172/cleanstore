use std::{fmt, fs, path::PathBuf};

#[derive(Debug, Default)]
struct DiskSpace(u64);

impl DiskSpace {
    /// Contrust a new container that counts no data.
    pub fn new() -> DiskSpace {
        DiskSpace::default()
    }

    /// Adds the amount of data to the thing
    pub fn add(&mut self, data: u64) {
        self.0 += data;
    }
}

impl fmt::Display for DiskSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = self.0 as f64;
        let mut u = 0_u8;

        while v > 1024.0 {
            v /= 1024.0;
            u += 1;
        }

        match u {
            0 => write!(f, "{v:.0}B"),
            1 => write!(f, "{v:.3}KB"),
            2 => write!(f, "{v:.3}MB"),
            3 => write!(f, "{v:.3}GB"),
            4 => write!(f, "{v:.3}TB"),
            _ => panic!("How big is your drive?"),
        }
    }
}

const MAX_DEPTH: usize = 10;

#[derive(Debug)]
pub struct SearchItem {
    path: PathBuf,
    depth: usize,
}

fn main() {
    let mut log = paris::Logger::new();

    // ------------------------------------------------------------------------------------------------------------

    let args = std::env::args();
    let path = args
        .skip(1)
        .next()
        .and_then(|s| fs::canonicalize(s).ok())
        .unwrap_or(PathBuf::from(std::env::var("HOME").unwrap()));

    log.info(format!("Starting from root dir: {}", path.display()));

    let root = SearchItem { path, depth: 0 };

    // ------------------------------------------------------------------------------------------------------------

    log.info("<green><bold>[+]</> <yellow>Removing .DS_Store files...</>");

    let mut removed: DiskSpace = DiskSpace::new();
    let mut q = vec![root];

    // iterate through all the files in the home directory of the user and remove the files that are named exactly '.DS_Store'
    // when each of the files are removed there size should be added to the amount_removed variable
    // the amount_removed variable should be converted to the largest unit possible
    while let Some(dir) = q.pop() {
        let r = match std::fs::read_dir(dir.path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for item in r {
            let item = item.unwrap();
            let path = item.path();
            let name = path.file_name().unwrap().to_str().unwrap();

            if dir.depth < MAX_DEPTH && path.is_dir()
            // I dont know if this should be included or not
            // && !name.starts_with('.')
            {
                // log.success(format!("checking dir: {}", path.display()));
                q.push(SearchItem {
                    path,
                    depth: dir.depth + 1,
                });
            } else if name == ".DS_Store" {
                // get the size of the file and add it to the amount_removed variable

                let size = path.metadata().unwrap().len();
                removed.add(size);

                log.log(format!(
                    "<red>(-)</> Removing file: {} <yellow>({} bytes)</>",
                    path.display(),
                    size
                ));

                std::fs::remove_file(path).unwrap();
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------

    log.success(format!("Liberated a total of {} bytes!</>", removed));
}
