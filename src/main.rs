use std::process;

use spotlight_save::Config;

fn main() {
    let config = Config::new().unwrap_or_else(|err| {
        eprintln!("Error when parsing arguments: {}", err);
        process::exit(1);
    });

    println!("{}", config)
}
