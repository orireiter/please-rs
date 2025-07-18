use std::io::Write;
use std::process;
use std::str::SplitWhitespace;
use std::{
    env::{self, current_dir},
    fmt::Display,
};

use anyhow::{Context, Result};

pub struct LiveCommand {
    pub user_command: Vec<char>,
}

impl Display for LiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match current_dir() {
            Ok(dir) => write!(f, "{}", dir.display()),
            Err(e) => write!(f, "<error: {e}>"),
        }
    }
}

impl LiveCommand {
    pub fn new() -> Self {
        Self {
            user_command: Vec::new(),
        }
    }

    pub fn execute_user_command(&mut self) -> Result<CommandOutcome> {
        let mut stdout = std::io::stdout();
        println!();
        stdout.flush()?;

        if self.user_command.is_empty() {
            return Ok(CommandOutcome::Continue);
        }

        let command_as_string = self.user_command.iter().collect::<String>();
        self.user_command.clear();

        if command_as_string == "\n" {
            return Ok(CommandOutcome::Continue);
        }

        let mut splitted_command = command_as_string.split_whitespace();

        let executable = if let Some(content) = splitted_command.next() {
            content
        } else {
            return Ok(CommandOutcome::Continue);
        };

        if executable == PleaseCommand::EXECUTABLE_NAME {
            let please_command = PleaseCommand::try_from(splitted_command)?;
            return please_command.handle_please_command();
        }

        let base_command = &mut self.get_base_process_command(executable).context(format!(
            "failed to build base command for {executable} with {splitted_command:?}"
        ))?;

        let user_command = base_command.args(splitted_command);

        let mut output = user_command.spawn()?;
        output.wait()?;
        stdout.flush()?;

        Ok(CommandOutcome::Continue)
    }

    fn get_base_process_command(&self, executable: &str) -> Result<process::Command> {
        // std::fs::metadata(path)
        // check if first arg exists in current dir or in PATH to see in need to prefix with cmd

        let _exectuable_metadata = std::fs::metadata(executable)
            .or_else(|_| {
                // let executable_as_string = executable.to_string();
                let path_values = env::var("PATH").map_err(|_| "").unwrap();

                path_values
                    .split(";")
                    .filter_map(|possible_path| {
                        std::fs::metadata(possible_path.to_string() + executable).ok()
                    })
                    .next()
                    .ok_or(|| "no exectuable in any place in PATH")
            })
            .map_err(|_e| anyhow::anyhow!(""));

        todo!("finish implementing")
    }
}

enum PleaseCommand {
    Exit,
}

impl PleaseCommand {
    pub const EXECUTABLE_NAME: &str = "please";

    fn handle_please_command(self) -> Result<CommandOutcome> {
        match self {
            Self::Exit => Ok(CommandOutcome::Close),
        }
    }
}

impl<'a> TryFrom<SplitWhitespace<'a>> for PleaseCommand {
    type Error = anyhow::Error;

    fn try_from(mut value: SplitWhitespace) -> std::result::Result<Self, Self::Error> {
        let main_arg = if let Some(content) = value.next() {
            content
        } else {
            return Err(anyhow::anyhow!("no main argument supplied for please"));
        };

        match main_arg {
            "exit" => Ok(Self::Exit),
            _ => Err(anyhow::anyhow!(
                "unknown please command argument {:?}",
                value
            )),
        }
    }
}

pub enum CommandOutcome {
    Continue,
    Close,
}
