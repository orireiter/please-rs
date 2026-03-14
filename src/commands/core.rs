use std::env::{self, current_dir};
use std::io::Write;
use std::process;
use std::str::SplitWhitespace;

use anyhow::{Context, Result};

use crate::utils::SPACE;

const CMD: &str = "cmd";
const RESERVED_CMD_COMMANDS: [&str; 82] = [
    "ASSOC",
    "ATTRIB",
    "BREAK",
    "BCDEDIT",
    "CACLS",
    "CALL",
    // "CD", commented to substitute with rust fn
    "CHCP",
    "CHDIR",
    "CHKDSK",
    "CHKNTFS",
    "CLS",
    "CMD",
    "COLOR",
    "COMP",
    "COMPACT",
    "CONVERT",
    "COPY",
    "DATE",
    "DEL",
    "DIR",
    "DISKPART",
    "DOSKEY",
    "DRIVERQUER",
    "ECHO",
    "ENDLOCAL",
    "ERASE",
    "EXIT",
    "FC",
    "FIND",
    "FINDSTR",
    "FOR",
    "FORMAT",
    "FSUTIL",
    "FTYPE",
    "GOTO",
    "GPRESULT",
    "HELP",
    "ICACLS",
    "IF",
    "LABEL",
    "MD",
    "MKDIR",
    "MKLINK",
    "MODE",
    "MORE",
    "MOVE",
    "OPENFILES",
    "PATH",
    "PAUSE",
    "POPD",
    "PRINT",
    "PROMPT",
    "PUSHD",
    "RD",
    "RECOVER",
    "REM",
    "REN",
    "RENAME",
    "REPLACE",
    "RMDIR",
    "ROBOCOPY",
    "SET",
    "SETLOCAL",
    "SC",
    "SCHTASKS",
    "SHIFT",
    "SHUTDOWN",
    "SORT",
    "START",
    "SUBST",
    "SYSTEMINFO",
    "TASKLIST",
    "TASKKILL",
    "TIME",
    "TITLE",
    "TREE",
    "TYPE",
    "VER",
    "VERIFY",
    "VOL",
    "XCOPY",
    "WMIC",
];
const KNOWN_CMD_EXECUTABLE_FILE_EXTENSIONS: [&str; 3] = ["", ".exe", ".bat"];

const PATH_ENV_VAR: &str = "PATH";
const PATH_ENV_VAR_DELIMITER: &str = ";";
const QUOTES: [&str; 2] = ["\"", "'"];

const LIVE_COMMAND_PREFIX_DELIMITER: &str = " -> ";

pub struct LiveCommand {
    pub user_command: Vec<char>,
}

impl LiveCommand {
    pub fn new() -> Self {
        Self {
            user_command: Vec::new(),
        }
    }

    pub fn execute_user_command(&mut self) -> Result<CommandOutcome> {
        let mut stdout = std::io::stdout();

        if self.user_command.is_empty() {
            return Ok(CommandOutcome::Skip);
        }

        let command_as_string = self.user_command_as_string();
        self.user_command.clear();

        if command_as_string == crate::utils::NEWLINE {
            return Ok(CommandOutcome::Continue);
        }

        let mut splitted_command = command_as_string.split_whitespace();

        let executable = if let Some(content) = splitted_command.next() {
            content
        } else {
            return Ok(CommandOutcome::Continue);
        };

        if PleaseCommand::is_please_command(executable) {
            let please_command = PleaseCommand::try_from(splitted_command)?;
            return please_command.execute_command();
        } else if let Ok(native_command) =
            NativeCommand::try_from_executable_and_args(executable, splitted_command.clone())
        {
            return native_command.execute_command();
        }

        let user_command = &mut self.get_base_process_command(executable).context(format!(
            "failed to build base command for {executable} with {splitted_command:?}"
        ))?;

        let args = self.get_process_args(splitted_command);
        user_command.args(args);

        crossterm::terminal::disable_raw_mode()?;

        if let Err(e) = user_command.spawn().and_then(|mut c| c.wait()) {
            log::error!("\"{command_as_string}\" {e}");
        };

        stdout.flush()?;

        crossterm::terminal::enable_raw_mode()?;

        Ok(CommandOutcome::Continue)
    }

    pub fn user_command_as_string(&self) -> String {
        self.user_command.iter().collect::<String>()
    }

    pub fn live_command_prefix(&self) -> String {
        let dir_part = match current_dir() {
            Ok(dir) => dir.display().to_string(),
            Err(e) => format!("<error: {e}>"),
        };

        let delimiter = LIVE_COMMAND_PREFIX_DELIMITER;

        format!("{dir_part}{delimiter}")
    }

    fn get_base_process_command(&self, executable: &str) -> Result<std::process::Command> {
        if std::fs::metadata(executable).is_ok() {
            return Ok(process::Command::new(executable));
        }

        if RESERVED_CMD_COMMANDS.contains(&executable.to_uppercase().as_str()) {
            let mut cmd = process::Command::new(CMD);
            cmd.arg("/c").arg(executable);
            return Ok(cmd);
        }

        let path_values = env::var(PATH_ENV_VAR)?;

        for possible_path in path_values.split(PATH_ENV_VAR_DELIMITER) {
            let temp_executable_path = format!("{possible_path}\\{executable}");

            for file_extension in KNOWN_CMD_EXECUTABLE_FILE_EXTENSIONS {
                if std::fs::metadata(format!("{temp_executable_path}{file_extension}")).is_ok() {
                    return Ok(process::Command::new(temp_executable_path));
                }
            }
        }

        Ok(process::Command::new(executable))
    }

    fn get_process_args(&self, split_string: SplitWhitespace) -> Vec<String> {
        let mut args = Vec::new();

        let mut quotes_used = None;
        let mut current_arg = String::new();
        for arg in split_string {
            if let Some(o) = QUOTES.iter().find(|q| arg.starts_with(*q))
                && quotes_used.is_none()
            {
                quotes_used = Some(*o);
            }

            if let Some(q) = quotes_used {
                let prefix = if current_arg.is_empty() { "" } else { SPACE };

                current_arg.push_str(prefix);
                current_arg.push_str(arg);
                if arg.ends_with(q) {
                    let to_push = current_arg
                        .strip_prefix(q)
                        .and_then(|a| a.strip_suffix(q))
                        .unwrap_or(&current_arg);

                    args.push(to_push.to_string());
                    current_arg.clear();
                    quotes_used = None;
                }
            } else {
                args.push(arg.to_string());
            }
        }

        args
    }

    pub fn get_latest_word(&self) -> String {
        let start = self
            .user_command
            .iter()
            .rposition(|&c| c == ' ')
            .map(|i| i + 1)
            .unwrap_or(0);

        self.user_command[start..].iter().collect()
    }
}

trait CommandExecution {
    fn execute_command(&self) -> Result<CommandOutcome>;
}

enum PleaseCommand {
    Exit,
}

impl PleaseCommand {
    const EXECUTABLE_NAME: &str = "please";
    const EXIT: &str = "exit";

    fn is_please_command(executable: &str) -> bool {
        executable == Self::EXECUTABLE_NAME
    }
}

impl CommandExecution for PleaseCommand {
    fn execute_command(&self) -> Result<CommandOutcome> {
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
            Self::EXIT => Ok(Self::Exit),
            _ => Err(anyhow::anyhow!(
                "unknown please command argument {:?}",
                value
            )),
        }
    }
}

enum NativeCommand {
    Clear,
    Ls(String),
    ChangeDir(String),
}

impl CommandExecution for NativeCommand {
    fn execute_command(&self) -> Result<CommandOutcome> {
        match self {
            Self::Clear => {
                let clear_options =
                    crate::utils::ClearOptions::new(crossterm::terminal::ClearType::Purge);
                crate::utils::clear_terminal(Some(clear_options))?;
                Ok(CommandOutcome::Continue)
            }
            Self::Ls(path) => {
                let path = if path.is_empty() { "." } else { path };

                process::Command::new(CMD)
                    .arg("/c")
                    .arg("dir")
                    .arg(path)
                    .spawn()?
                    .wait()?;
                Ok(CommandOutcome::Continue)
            }
            Self::ChangeDir(new_dir) => {
                std::env::set_current_dir(new_dir)?;
                Ok(CommandOutcome::Continue)
            }
        }
    }
}

impl NativeCommand {
    const CLEAR: &str = "clear";
    const LS: &str = "ls";
    const CD: &str = "cd";
    const CHDIR: &str = "chdir";

    fn try_from_executable_and_args(executable: &str, args: SplitWhitespace) -> Result<Self> {
        match executable.to_lowercase().as_str() {
            Self::CLEAR => Ok(Self::Clear),
            Self::LS => Ok(Self::Ls(args.collect())),
            Self::CD | Self::CHDIR => Ok(Self::ChangeDir(args.collect())),
            _ => Err(anyhow::anyhow!(
                "unknown native command \"{executable}\" with args {:?}",
                args
            )),
        }
    }
}

pub enum CommandOutcome {
    Continue,
    Close,
    Skip,
}
