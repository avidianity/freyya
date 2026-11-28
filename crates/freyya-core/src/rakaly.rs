use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::Refusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rakaly {
    program: PathBuf,
}

impl Rakaly {
    pub fn on_path() -> Self {
        Self::at("rakaly")
    }

    pub fn at(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    pub(crate) fn melt(&self, save: &[u8]) -> Result<Vec<u8>, Refusal> {
        let mut child = Command::new(&self.program)
            .args([
                "melt",
                "--to-stdout",
                "--unknown-key",
                "error",
                "--format",
                "vic3",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| Refusal::ReaderUnavailable {
                detail: format!("{}: {error}", self.program.display()),
            })?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Refusal::ReaderUnavailable {
                detail: "no input pipe to the reader".to_owned(),
            })?;
        let (fed, output) = std::thread::scope(|scope| {
            let feeder = scope.spawn(move || stdin.write_all(save));
            let output = child.wait_with_output();
            (feeder.join(), output)
        });
        let output = output.map_err(|error| Refusal::ReaderUnavailable {
            detail: error.to_string(),
        })?;
        if !output.status.success() {
            return Err(Refusal::Damaged {
                detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        match fed {
            Ok(Ok(())) => Ok(output.stdout),
            Ok(Err(error)) => Err(Refusal::ReaderUnavailable {
                detail: format!("could not pass the Save to the reader: {error}"),
            }),
            Err(_) => Err(Refusal::ReaderUnavailable {
                detail: "the thread passing the Save to the reader panicked".to_owned(),
            }),
        }
    }
}
