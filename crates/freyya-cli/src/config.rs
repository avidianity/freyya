use std::path::PathBuf;

use freyya_core::Rakaly;

pub(crate) const USAGE: &str = "usage: freyya-cli read-save <SAVE> [--rakaly <PATH>] [--game <DIR>]\n       freyya-cli check-game-files [--game <DIR>]";

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
}

pub(crate) fn command_from_args(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    match args.next().as_deref() {
        Some("read-save") => read_save_args(args),
        Some("check-game-files") => check_game_files_args(args),
        Some(other) => Err(format!("unknown command {other:?}")),
        None => Err("no command given".to_owned()),
    }
}

fn read_save_args(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut save = None;
    let mut rakaly = Rakaly::on_path();
    let mut game = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rakaly" => {
                let path = args.next().ok_or("--rakaly needs a path")?;
                rakaly = Rakaly::at(path);
            }
            "--game" => game = Some(game_folder_arg(&mut args)?),
            _ if save.is_none() => save = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument {arg:?}")),
        }
    }
    let save = save.ok_or("no Save path given")?;
    Ok(Command::ReadSave { save, rakaly, game })
}

fn check_game_files_args(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut game = PathBuf::from(DEFAULT_GAME_FOLDER);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => game = game_folder_arg(&mut args)?,
            _ => return Err(format!("unexpected argument {arg:?}")),
        }
    }
    Ok(Command::CheckGameFiles { game })
}

fn game_folder_arg(args: &mut impl Iterator<Item = String>) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| "--game needs the Victoria 3 install folder".to_owned())
}
