use std::{fmt::Display, path::PathBuf};

use clap::{App, Arg};

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
        let matches = App::new("spotlight_save")
                        .version("1.0.0")
                        .author("Steven Lee <leexgone@163.com>")
                        .about("Save splotlight images in Win10.")
                        .arg(Arg::with_name("DIR")
                            .help("Target image dir. Default dir is '${HOME}/Pictures/Spotlight/'")
                            .index(1))
                        .arg(Arg::with_name("verbose")
                            .short("v")
                            .help("Use verbose output"))
                        .arg(Arg::with_name("archive")
                            .short("a")
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