#![doc = "madobe command-line control executable."]
#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match madobectl::run(env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}
