use std::path::PathBuf;

use freyya_core::Rakaly;

pub(crate) const USAGE: &str = "usage: freyya-cli read-save <SAVE> [--rakaly <PATH>]";

#[derive(Debug)]
pub(crate) enum Command {
    ReadSave { save: PathBuf, rakaly: Rakaly },
}

pub(crate) fn command_from_args(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    match args.next().as_deref() {
        Some("read-save") => read_save_args(args),
        Some(other) => Err(format!("unknown command {other:?}")),
        None => Err("no command given".to_owned()),
    }
}

fn read_save_args(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut save = None;
    let mut rakaly = Rakaly::on_path();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rakaly" => {
                let path = args.next().ok_or("--rakaly needs a path")?;
                rakaly = Rakaly::at(path);
            }
            _ if save.is_none() => save = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument {arg:?}")),
        }
    }
    let save = save.ok_or("no Save path given")?;
    Ok(Command::ReadSave { save, rakaly })
}
