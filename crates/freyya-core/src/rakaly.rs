use std::any::Any;
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
            .map_err(|error| {
                Refusal::reader_unavailable_by(self.program.display().to_string(), error)
            })?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Refusal::reader_unavailable("no input pipe to the reader"))?;
        let (fed, output) = std::thread::scope(|scope| {
            let feeder = scope.spawn(move || stdin.write_all(save));
            let output = child.wait_with_output();
            (feeder.join(), output)
        });
        let output = output.map_err(|error| {
            Refusal::reader_unavailable_by("could not collect the reader's output", error)
        })?;
        if !output.status.success() {
            return Err(Refusal::damaged(
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        match fed {
            Ok(Ok(())) => Ok(output.stdout),
            Ok(Err(error)) => Err(Refusal::reader_unavailable_by(
                "could not pass the Save to the reader",
                error,
            )),
            Err(panic) => Err(Refusal::reader_unavailable(format!(
                "the thread passing the Save to the reader panicked: {}",
                panic_message(panic.as_ref())
            ))),
        }
    }
}

fn panic_message(panic: &(dyn Any + Send)) -> &str {
    if let Some(message) = panic.downcast_ref::<&str>() {
        message
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message
    } else {
        "no message"
    }
}
