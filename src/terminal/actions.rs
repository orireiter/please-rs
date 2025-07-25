use crossterm::event::{
    Event as CrosstermEvent, KeyCode as CrosstermKeyCode, KeyEvent as CrosstermKeyEvent,
};

pub enum ActionType {
    UserAction,
    HistoryAction,
}

pub struct Action {
    pub _action_type: ActionType,
    pub event: CrosstermEvent,
}

impl Action {
    pub fn new_user_action(event: CrosstermEvent) -> Self {
        Self {
            _action_type: ActionType::UserAction,
            event,
        }
    }

    pub fn new_history_action(event: CrosstermEvent) -> Self {
        Self {
            _action_type: ActionType::HistoryAction,
            event,
        }
    }

    pub fn new_history_key_pressed_action(char_pressed: char) -> Self {
        let key_code = CrosstermKeyCode::Char(char_pressed);
        let key_event = CrosstermKeyEvent::from(key_code);
        let event = CrosstermEvent::Key(key_event);

        Self::new_history_action(event)
    }
}
