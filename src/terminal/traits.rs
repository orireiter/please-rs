use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::commands;

pub trait KeyHandling {
    fn handle_enter(&mut self, stdout: &mut std::io::Stdout) -> commands::CommandOutcome;

    fn handle_backspace(&mut self, stdout: &mut std::io::Stdout) -> Result<()>;

    fn handle_up(&mut self, stdout: &mut std::io::Stdout) -> Result<()>;

    fn handle_down(&mut self, stdout: &mut std::io::Stdout) -> Result<()>;

    fn handle_left(&mut self, stdout: &mut std::io::Stdout, key_event: KeyEvent) -> Result<()>;

    fn handle_right(&mut self, stdout: &mut std::io::Stdout, key_event: KeyEvent) -> Result<()>;
}
