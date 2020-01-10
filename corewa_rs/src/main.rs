extern crate corewa_rs;

use corewa_rs::cli;

fn main() {
    std::process::exit(
        // TODO use exitcode lib or something like that
        if let Err(err) = cli::run() {
            eprintln!("Error: {}", err);
            -1
        } else {
            // TODO use exit codes for warnings?
            0
        },
    )
}
