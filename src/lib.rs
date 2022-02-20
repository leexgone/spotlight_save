use std::{error::Error, fmt::Display, fs, path::PathBuf};

use chrono::{DateTime, Duration, Local};
use clap::{Command, Arg};
use image::{io::Reader};

#[derive(Debug)]
pub struct Config {
    target: PathBuf,
    verbose: bool,
    archive: bool,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[target = {}, verbose = {}, archive = {}]", self.target.display(), self.verbose, self.archive)
    }
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let matches = Command::new("spotlight_save")
                        .version("1.0.0")
                        .author("Steven Lee <leexgone@163.com>")
                        .about("Save splotlight images in Win10.")
                        .arg(Arg::new("DIR")
                            .help("Target image dir. Default dir is '${HOME}/Pictures/Spotlight/'")
                            .index(1))
                        .arg(Arg::new("verbose")
                            .short('v')
                            .long("verbose")
                            .help("Use verbose output"))
                        .arg(Arg::new("archive")
                            .short('a')
                            .long("archive")
                            .help("Archive images by year"))
                        .get_matches();

        let verbose = matches.is_present("verbose");
        let archive = matches.is_present("archive");

        let target = if let Some(dir) = matches.value_of("DIR") {
            PathBuf::from(dir)
        } else {
            let home_dir = home::home_dir().unwrap();
            let picture_dir = home_dir.join("Pictures");
            if !picture_dir.is_dir() {
                let msg = format!("Can not find dir '{}'", picture_dir.display());
                return Err(msg);
            }

            picture_dir.join("Spotlight")
        };

        if !target.is_dir() {
            let msg = format!("target dir '{}' does not exist", target.display());
            return Err(msg);
        }

        Ok(Config {
            target,
            verbose,
            archive,
        })
    }
}

macro_rules! log {
    ($enabled:expr) => {
        {if $enabled { println!(); }}
    };
    ($enabled:expr, $($arg:tt)*) => {
        {if $enabled { println!($($arg)*); }}
    };
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    save_images(&config)?;
    if config.archive {
        archive_images(&config)?;
    }

    Ok(())
}

fn get_spotlight_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home_dir = home::home_dir().unwrap();
    let package_dir = home_dir.join("AppData\\Local\\Packages");

    for entry in package_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let pathname = path.file_name().unwrap();
        let pathname = pathname.to_str().unwrap();

        if pathname.starts_with("Microsoft.Windows.ContentDeliveryManager_") {
            let image_dir = path.join("LocalState\\Assets");

            return  Ok(image_dir);
        }
    }

    let err = std::io::Error::new(std::io::ErrorKind::NotFound, String::from("Can not find Spotlight image dir"));
    Err(Box::new(err))
}

fn save_images(config: &Config) -> Result<(), Box<dyn Error>> {
    let spotlight_dir = get_spotlight_dir()?;
    log!(config.verbose, "Scan spotlight dir: {}", spotlight_dir.display());

    let mut count = 0;
    for entry in spotlight_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if save_image(config, &path) {
            count += 1;
        }
    }

    log!(config.verbose, "{} images saved!", count);

    Ok(())
}

fn save_image(config: &Config, filepath: &PathBuf) -> bool {
    log!(config.verbose, "Scan file: {}...", filepath.display());

    let reader = if let Ok(reader) = Reader::open(filepath) {
        reader
    } else {
        return false;
    };
    let reader = if let Ok(reader) = reader.with_guessed_format() {
        reader
    } else {
        return false;
    };
    let format = if let Some(format) = reader.format() {
        format
    } else {
        return false;
    };
    let image = if let Ok(image) = reader.decode() {
        image
    } else {
        return false;
    };

    if image.width() < image.height() || image.width() < 800 || image.height() < 600 {
        return false;
    }

    let ext = format.extensions_str().first().unwrap();
    let mut filename = String::from(filepath.file_name().unwrap().to_str().unwrap());
    filename.push_str(".");
    filename.push_str(*ext);

    let target_file = config.target.join(filename);
    if target_file.exists() {
        return false;
    }

    log!(config.verbose, "Saving image: {} ...", target_file.display());

    fs::copy(filepath, target_file).is_ok()
}

fn archive_images(config: &Config) -> Result<(), Box<dyn Error>> {
    log!(config.verbose, "Archive images in dir: {}", config.target.display());

    let timeline = Local::today() - Duration::days(365);

    for entry in config.target.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Ok(metadata) = entry.metadata() {
            let filetime = if let Ok(modified) = metadata.modified() {
                modified
            } else if let Ok(created) = metadata.created() {
                created
            } else {
                continue;
            };
            let filedate = DateTime::from(filetime).date();

            if filedate < timeline {    
                log!(config.verbose, "archive file: {} ...", path.display());
                
                let year = filedate.format("%Y").to_string();
                let dir = config.target.join(year);
                if !dir.exists() {
                    fs::create_dir(&dir)?;
                }
                let bak_file = dir.join(path.file_name().unwrap());
                fs::copy(&path, &bak_file)?;
                fs::remove_file(&path)?;
            }
        }
    }

    Ok(())
}
