use std::env::{self};
use std::io::Write;
use std::str::{FromStr, SplitWhitespace};
use std::{path, process};

use anyhow::{Context, Result};

use crate::commands::config::CommandConfig;
use crate::commands::prefix::LiveCommandPrefix;
use crate::utils::{SPACE, StyledContentGroup};

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
const KNOWN_CMD_EXECUTABLE_FILE_EXTENSIONS: [&str; 4] = ["exe", "bat", "cmd", ""];

const PATH_ENV_VAR: &str = "PATH";
const PATH_ENV_VAR_DELIMITER: &str = ";";
const QUOTES: [&str; 2] = ["\"", "'"];

pub struct LiveCommand {
    pub user_command: Vec<char>,
    command_prefix: LiveCommandPrefix,
    #[allow(dead_code)]
    config: CommandConfig,
}

impl LiveCommand {
    pub fn from_config(config: CommandConfig) -> Self {
        Self {
            user_command: Vec::new(),
            command_prefix: LiveCommandPrefix::from_config(config.prefix_config.clone()),
            config,
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

        if please::PleaseCommand::is_please_command(executable) {
            let please_command = please::PleaseCommand::try_from(splitted_command)?;
            return please_command.execute_command();
        } else if let Ok(native_command) = native::NativeCommand::try_from_executable_and_args(
            executable,
            splitted_command.clone(),
        ) {
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

    pub fn live_command_prefix(&self) -> StyledContentGroup {
        self.command_prefix.get_command_prefix()
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
            let possible_env_path = path::PathBuf::from_str(possible_path);
            let mut temp_executable_path = match possible_env_path {
                Ok(env_path) => env_path.join(executable),
                Err(_) => continue,
            };

            for file_extension in KNOWN_CMD_EXECUTABLE_FILE_EXTENSIONS {
                temp_executable_path.set_extension(file_extension);
                if temp_executable_path.metadata().is_ok() {
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

    pub fn get_full_len(&self) -> usize {
        self.live_command_prefix().len() + self.user_command.len()
    }

    // todo handle env vars before executable
}

trait CommandExecution {
    fn execute_command(&self) -> Result<CommandOutcome>;
}

pub mod please {
    use std::{collections::HashMap, str::SplitWhitespace, sync::LazyLock};

    use anyhow::Result;

    use crate::commands::{CommandOutcome, core::CommandExecution};

    #[derive(Clone)]
    pub enum PleaseCommand {
        Exit,
        Reload,
    }

    pub static PLEASE_COMMANDS_MAP: LazyLock<HashMap<&'static str, PleaseCommand>> =
        LazyLock::new(|| {
            HashMap::from([
                ("exit", PleaseCommand::Exit),
                ("reload", PleaseCommand::Reload),
            ])
        });

    impl PleaseCommand {
        pub const EXECUTABLE_NAME: &str = "please";

        pub fn is_please_command(executable: &str) -> bool {
            executable.eq_ignore_ascii_case(Self::EXECUTABLE_NAME)
        }
    }

    impl CommandExecution for PleaseCommand {
        fn execute_command(&self) -> Result<CommandOutcome> {
            match self {
                Self::Exit => Ok(CommandOutcome::Close),
                Self::Reload => Ok(CommandOutcome::Reload),
            }
        }
    }

    impl<'a> TryFrom<SplitWhitespace<'a>> for PleaseCommand {
        type Error = anyhow::Error;

        fn try_from(mut value: SplitWhitespace) -> std::result::Result<Self, Self::Error> {
            let main_arg = if let Some(content) = value.next() {
                content.to_lowercase()
            } else {
                return Err(anyhow::anyhow!("no main argument supplied for please"));
            };

            match PLEASE_COMMANDS_MAP.get(main_arg.as_str()) {
                Some(cmd) => Ok(cmd.clone()),
                None => Err(anyhow::anyhow!(
                    "unknown please command argument {:?}",
                    value
                )),
            }
        }
    }
}

pub mod native {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        process,
        str::SplitWhitespace,
    };

    use anyhow::Result;

    use crate::commands::{
        CommandOutcome,
        core::{CMD, CommandExecution},
    };

    pub enum NativeCommand {
        Clear,
        Ls(String),
        ChangeDir(String),
        Cat(String),
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
                Self::Cat(path) => {
                    if path.is_empty() {
                        Err(anyhow::anyhow!("no file specified"))
                    } else {
                        let file = File::open(path)?;
                        let buf_read = BufReader::new(file);
                        for line in buf_read.lines() {
                            println!("{}", line?);
                        }
                        println!();

                        Ok(CommandOutcome::Continue)
                    }
                }
            }
        }
    }

    impl NativeCommand {
        const CLEAR: &str = "clear";
        const LS: &str = "ls";
        const CD: &str = "cd";
        const CHDIR: &str = "chdir";
        const CAT: &str = "cat";

        pub fn try_from_executable_and_args(
            executable: &str,
            args: SplitWhitespace,
        ) -> Result<Self> {
            match executable.to_lowercase().as_str() {
                Self::CLEAR => Ok(Self::Clear),
                Self::LS => Ok(Self::Ls(args.collect())),
                Self::CD | Self::CHDIR => Ok(Self::ChangeDir(args.collect())),
                Self::CAT => {
                    let mut cloned_args = args.clone();
                    let path = cloned_args.next().unwrap_or_default();
                    if cloned_args.next().is_some() {
                        return Err(anyhow::anyhow!("cat accepts only a single path argument"));
                    }
                    Ok(Self::Cat(path.to_string()))
                }
                _ => Err(anyhow::anyhow!(
                    "unknown native command \"{executable}\" with args {:?}",
                    args
                )),
            }
        }
    }
}

pub enum CommandOutcome {
    Continue,
    Close,
    Reload,
    Skip,
}
