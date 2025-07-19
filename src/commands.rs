use std::env::{self, current_dir};
use std::io::Write;
use std::process;
use std::str::SplitWhitespace;

use anyhow::{Context, Result};

const CMD: &str = "cmd";
const RESERVED_CMD_COMMANDS: [&str; 83] = [
    "ASSOC",
    "ATTRIB",
    "BREAK",
    "BCDEDIT",
    "CACLS",
    "CALL",
    "CD",
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
            return Ok(CommandOutcome::Continue);
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

        user_command.args(splitted_command);

        let mut output = user_command.spawn()?;
        output.wait()?;
        stdout.flush()?;

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
            Self::Ls(_path) => {
                todo!("implement \"ls\"")
            }
        }
    }
}

impl NativeCommand {
    const CLEAR: &str = "clear";
    const LS: &str = "ls";

    fn try_from_executable_and_args(executable: &str, args: SplitWhitespace) -> Result<Self> {
        match executable.to_lowercase().as_str() {
            Self::CLEAR => Ok(Self::Clear),
            Self::LS => Ok(Self::Ls(args.collect())),
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
}
