mod buildings_jobs;
mod config;

use std::path::Path;
use std::process::ExitCode;

use freyya_core::{Rakaly, read_save};

use config::{Command, USAGE, command_from_args};

fn main() -> ExitCode {
    match command_from_args(std::env::args().skip(1)) {
        Ok(Command::ReadSave { save, rakaly }) => print_save(&save, &rakaly),
        Err(problem) => {
            eprintln!("{problem}\n{USAGE}");
            ExitCode::from(64)
        }
    }
}

fn print_save(path: &Path, rakaly: &Rakaly) -> ExitCode {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", path.display());
            return ExitCode::from(66);
        }
    };
    match read_save(&bytes, rakaly) {
        Ok(snapshot) => {
            println!("Save:           {}", path.display());
            println!("Game version:   {}", snapshot.game_version);
            println!("Game date:      {}", snapshot.game_date);
            println!(
                "Player country: {} ({})",
                snapshot.player_country.name, snapshot.player_country.tag
            );
            buildings_jobs::print_buildings_and_jobs(&snapshot.buildings_and_jobs);
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("Refused {}: {refusal}", path.display());
            ExitCode::from(65)
        }
    }
}
