#![doc = "madobe command-line control executable."]
#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if matches!(args.first().map(String::as_str), Some("video-smoke")) {
        return run_video_smoke(&args[1..]);
    }

    let output = match madobectl::requires_compositor_adapter(args.iter().map(String::as_str)) {
        Ok(true) => {
            let mut adapter =
                match madobe_hyprland::HyprlandAdapter::new(madobe_hyprland::HyprctlExecutor) {
                    Ok(adapter) => adapter,
                    Err(error) => {
                        eprintln!("invalid display configuration: {error}");
                        return ExitCode::from(2);
                    }
                };

            madobectl::run_with_adapter(args.iter().map(String::as_str), &mut adapter)
        }
        Ok(false) => madobectl::run(args.iter().map(String::as_str)),
        Err(error) => Err(error),
    };

    match output {
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

fn run_video_smoke(args: &[String]) -> ExitCode {
    match madobe_transport::video_smoke::run_cli(args.iter().map(String::as_str)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
