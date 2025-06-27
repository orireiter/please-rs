use anyhow::{Ok, Result};
use crossterm::{
    QueueableCommand, cursor,
    event::{Event as TerminalEvent, read},
    terminal,
};
use std::io::{Write, stdout};
use std::process::Command as UserCommand;

fn print_events() -> Result<()> {
    let mut stdout = stdout();
    let mut accumulating_command = Vec::new();
    loop {
        let event = read()?;
        match event {
            TerminalEvent::Key(key_event) => {
                let as_char = key_event.code.as_char();
                if key_event.is_press() {
                    if let Some(c) = as_char {
                        accumulating_command.push(c);
                        print!("{c}");
                        stdout.flush()?;
                    } else if key_event.code.is_enter() {
                        println!();
                        stdout.flush()?;
                        let command_as_string = accumulating_command.iter().collect::<String>();
                        if command_as_string.is_empty() || command_as_string == "\n" {
                            continue;
                        } else if command_as_string == "exit" {
                            // todo save special commands in enum
                            return Ok(());
                        }

                        let mut splitted = command_as_string.split_whitespace();

                        if let Some(command_base) = splitted.next() {
                            let mut user_command = UserCommand::new(command_base);

                            for arg in splitted {
                                user_command.arg(arg);
                            }

                            let output = user_command.output()?;
                            stdout.write_all(&output.stdout)?;
                            stdout.flush()?;
                        }

                        todo!("Execute command")
                    } else if key_event.code.is_backspace() {
                        accumulating_command.pop();

                        let (x, y) = cursor::position()?;
                        if x == 0 && y == 0 {
                            continue;
                        } else if x > 0 {
                            stdout.queue(cursor::MoveLeft(1))?;
                            print!(" ");
                            stdout.queue(cursor::MoveLeft(1))?;
                        } else {
                            let terminal_size = terminal::size()?;
                            stdout.queue(cursor::MoveTo(terminal_size.0, y - 1))?;
                            print!(" ");
                            stdout.queue(cursor::MoveTo(terminal_size.0, y - 1))?;
                        }

                        stdout.flush()?;
                    }
                }
            }
            TerminalEvent::FocusGained => todo!(),
            TerminalEvent::FocusLost => todo!(),
            TerminalEvent::Mouse(_) => todo!(),
            TerminalEvent::Paste(_) => todo!(),
            TerminalEvent::Resize(_, _) => todo!(),
        }
    }
}

fn init_terminal() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;

    stdout
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(cursor::EnableBlinking)?
        .flush()?;

    Ok(())
}

fn main() -> Result<()> {
    init_terminal()?;
    print_events()?;

    Ok(())
}
