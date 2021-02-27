use std::process;

use spotlight_save::Config;

fn main() {
    let config = Config::new().unwrap_or_else(|err| {
        eprintln!("Error when parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = spotlight_save::run(config) {
        eprintln!("Error when saving images: {}", e);
        process::exit(1);
    }
}
