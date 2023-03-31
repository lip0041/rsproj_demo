use std::env;
use std::process;

use minigrep::Config;
fn main() {
    let config = Config::build(env::args().collect()).unwrap_or_else(|err| {
        eprintln!("problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("query string: {0}", config.query);
    println!("query file: {0}", config.file_path);

    if let Err(e) = minigrep::run(config) {
        eprintln!("application error: {e}");
        process::exit(1);
    }
}
