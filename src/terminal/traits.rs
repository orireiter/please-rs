use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::commands;

pub trait KeyHandling {
    fn handle_enter(&mut self, stdout: &mut std::io::Stdout) -> commands::CommandOutcome;

    fn handle_backspace(&mut self, stdout: &mut std::io::Stdout, key_event: KeyEvent)
    -> Result<()>;

    fn handle_up(&mut self, stdout: &mut std::io::Stdout) -> Result<()>;

    fn handle_down(&mut self, stdout: &mut std::io::Stdout) -> Result<()>;

    fn handle_left(&mut self, stdout: &mut std::io::Stdout, key_event: KeyEvent) -> Result<()>;

    fn handle_right(&mut self, stdout: &mut std::io::Stdout, key_event: KeyEvent) -> Result<()>;
}

#[allow(dead_code)]
pub trait IsKeyEvents {
    fn is_backspace_key_event(&self, key_event: crossterm::event::KeyEvent) -> bool;

    fn is_ctrl_c_key_event(&self, key_event: crossterm::event::KeyEvent) -> bool;
}
