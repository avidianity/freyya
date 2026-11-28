mod checks;
mod git;
mod markdown;
mod policy;
mod rust_source;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match checks::run(&args) {
        Ok(failures) if failures.is_empty() => ExitCode::SUCCESS,
        Ok(failures) => {
            for failure in &failures {
                eprintln!("{failure}");
            }
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("xtask: {error:#}");
            ExitCode::from(2)
        }
    }
}
