//! The responsibility: launch external programs for opening image files.

use std::process::{Command, Stdio};

use crate::config::raw_config::RawConfig;

/// Opens paths with a configured command template.
///
/// The command should include `<path>` where the image path belongs. If it does not, the default
/// opener is used to avoid launching a command that ignores the selected image.
pub struct ExternalOpener {
    open_command: Vec<String>,
}

impl ExternalOpener {
    /// Creates an opener from the current application configuration.
    pub fn new(open_command: Vec<String>) -> Self {
        Self { open_command }
    }

    /// Spawns the configured opener without attaching stdio to the current process.
    pub fn open(mut self, image_path: &str) -> eyre::Result<()> {
        let index = self.open_command.iter().position(|x| x == "<path>");

        let mut cmd = match index {
            Some(index) => {
                self.open_command[index] = image_path.to_string();
                let first_arg = self.open_command[0].clone();
                let mut cmd = Command::new(&first_arg);
                cmd.args(&self.open_command[1..]);
                cmd
            }
            None => {
                let app_config = RawConfig::default();
                let mut open_command = app_config.open_command;
                if let Some(index) = open_command.iter().position(|x| x == "<path>") {
                    open_command[index] = image_path.to_string();
                }
                let first_arg = open_command[0].clone();
                let mut cmd = Command::new(&first_arg);
                cmd.args(&open_command[1..]);
                cmd
            }
        };

        cmd.stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null());

        cmd.spawn()?;

        Ok(())
    }
}
