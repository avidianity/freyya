use std::fmt;
use std::path::{Path, PathBuf};

use freyya_core::Rakaly;

pub(crate) const USAGE: &str = "usage: freyya-cli read-save <SAVE> [--rakaly <PATH>] [--game <DIR>]\n       freyya-cli check-game-files [--game <DIR>]\n       freyya-cli measure-save <SAVE>... [--rakaly <PATH>]";

const DEFAULT_RAKALY: &str = "rakaly";

const DEFAULT_GAME_FOLDER: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Victoria 3";

#[derive(Debug)]
pub(crate) enum Command {
    ReadSave {
        save: PathBuf,
        rakaly: Rakaly,
        game: Option<PathBuf>,
    },
    CheckGameFiles {
        game: PathBuf,
    },
    MeasureSave {
        saves: Vec<PathBuf>,
        rakaly: Rakaly,
    },
}

#[derive(Debug)]
pub(crate) enum ConfigError {
    Usage(String),
    MissingPath(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Usage(problem) => write!(f, "{problem}\n{USAGE}"),
            ConfigError::MissingPath(problem) => f.write_str(problem),
        }
    }
}

impl From<String> for ConfigError {
    fn from(problem: String) -> Self {
        ConfigError::Usage(problem)
    }
}

impl From<&str> for ConfigError {
    fn from(problem: &str) -> Self {
        ConfigError::Usage(problem.to_owned())
    }
}

pub(crate) fn command_from_args(
    mut args: impl Iterator<Item = String>,
) -> Result<Command, ConfigError> {
    match args.next().as_deref() {
        Some("read-save") => read_save_args(args),
        Some("check-game-files") => check_game_files_args(args),
        Some("measure-save") => measure_save_args(args),
        Some(other) => Err(format!("unknown command {other:?}").into()),
        None => Err("no command given".into()),
    }
}

fn read_save_args(mut args: impl Iterator<Item = String>) -> Result<Command, ConfigError> {
    let mut save = None;
    let mut rakaly = PathBuf::from(DEFAULT_RAKALY);
    let mut game = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rakaly" => rakaly = rakaly_arg(&mut args)?,
            "--game" => game = Some(game_folder_arg(&mut args)?),
            _ if save.is_none() => save = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument {arg:?}").into()),
        }
    }
    let save = save.ok_or("no Save path given")?;
    Ok(Command::ReadSave {
        save,
        rakaly: checked_rakaly(rakaly)?,
        game: game.map(checked_game_folder).transpose()?,
    })
}

fn measure_save_args(mut args: impl Iterator<Item = String>) -> Result<Command, ConfigError> {
    let mut saves = Vec::new();
    let mut rakaly = PathBuf::from(DEFAULT_RAKALY);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rakaly" => rakaly = rakaly_arg(&mut args)?,
            _ => saves.push(PathBuf::from(arg)),
        }
    }
    if saves.is_empty() {
        return Err("no Save path given".into());
    }
    Ok(Command::MeasureSave {
        saves,
        rakaly: checked_rakaly(rakaly)?,
    })
}

fn check_game_files_args(mut args: impl Iterator<Item = String>) -> Result<Command, ConfigError> {
    let mut game = PathBuf::from(DEFAULT_GAME_FOLDER);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => game = game_folder_arg(&mut args)?,
            _ => return Err(format!("unexpected argument {arg:?}").into()),
        }
    }
    Ok(Command::CheckGameFiles {
        game: checked_game_folder(game)?,
    })
}

fn rakaly_arg(args: &mut impl Iterator<Item = String>) -> Result<PathBuf, ConfigError> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| "--rakaly needs a path".into())
}

fn game_folder_arg(args: &mut impl Iterator<Item = String>) -> Result<PathBuf, ConfigError> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| "--game needs the Victoria 3 install folder".into())
}

fn checked_rakaly(program: PathBuf) -> Result<Rakaly, ConfigError> {
    if program_exists(&program) {
        Ok(Rakaly::at(program))
    } else {
        Err(ConfigError::MissingPath(format!(
            "cannot find the Rakaly CLI at {}: install it on PATH or pass --rakaly <PATH>",
            program.display()
        )))
    }
}

fn checked_game_folder(folder: PathBuf) -> Result<PathBuf, ConfigError> {
    if folder.is_dir() {
        Ok(folder)
    } else {
        Err(ConfigError::MissingPath(format!(
            "cannot find the Victoria 3 install folder {}: pass --game <DIR>",
            folder.display()
        )))
    }
}

fn program_exists(program: &Path) -> bool {
    if program.components().count() > 1 {
        return program.is_file();
    }
    let candidates = [
        program.to_path_buf(),
        program.with_extension(std::env::consts::EXE_EXTENSION),
    ];
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths)
            .any(|folder| candidates.iter().any(|name| folder.join(name).is_file()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> impl Iterator<Item = String> {
        words
            .iter()
            .map(|word| (*word).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn missing_folder() -> PathBuf {
        std::env::temp_dir().join("freyya-cli-no-such-folder")
    }

    fn existing_program() -> String {
        std::env::current_exe()
            .expect("the test binary has a path")
            .display()
            .to_string()
    }

    #[test]
    fn read_save_stops_when_the_rakaly_path_does_not_exist() {
        let rakaly = missing_folder().join("rakaly.exe").display().to_string();

        let refusal = command_from_args(args(&["read-save", "a.v3", "--rakaly", &rakaly]))
            .expect_err("a missing Rakaly is refused");

        assert!(
            matches!(&refusal, ConfigError::MissingPath(problem) if problem.contains(&rakaly)),
            "got {refusal:?}"
        );
    }

    #[test]
    fn read_save_stops_when_the_game_folder_does_not_exist() {
        let game = missing_folder().display().to_string();
        let rakaly = existing_program();

        let refusal = command_from_args(args(&[
            "read-save",
            "a.v3",
            "--rakaly",
            &rakaly,
            "--game",
            &game,
        ]))
        .expect_err("a missing game folder is refused");

        assert!(
            matches!(&refusal, ConfigError::MissingPath(problem) if problem.contains(&game)),
            "got {refusal:?}"
        );
    }

    #[test]
    fn check_game_files_stops_when_the_game_folder_does_not_exist() {
        let game = missing_folder().display().to_string();

        let refusal = command_from_args(args(&["check-game-files", "--game", &game]))
            .expect_err("a missing game folder is refused");

        assert!(
            matches!(&refusal, ConfigError::MissingPath(problem) if problem.contains(&game)),
            "got {refusal:?}"
        );
    }

    #[test]
    fn read_save_starts_when_the_rakaly_and_game_paths_exist() {
        let rakaly = existing_program();
        let game = std::env::temp_dir().display().to_string();

        let command = command_from_args(args(&[
            "read-save",
            "a.v3",
            "--rakaly",
            &rakaly,
            "--game",
            &game,
        ]))
        .expect("existing paths are accepted");

        assert!(
            matches!(&command, Command::ReadSave { game: Some(folder), .. } if folder.display().to_string() == game),
            "got {command:?}"
        );
    }
}
