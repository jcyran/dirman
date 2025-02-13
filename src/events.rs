use crossterm::event::{KeyCode, KeyEvent};

pub enum MyEvent {
    MoveUp,
    MoveDown,
    Exit
}

pub struct AppEvents {
    
}

impl AppEvents {
    pub fn handle_key_event(key_event: KeyEvent) -> Option<MyEvent> {
        match key_event.code {
            KeyCode::Char('q') => Some(MyEvent::Exit),
            KeyCode::Up => Some(MyEvent::MoveUp),
            KeyCode::Down => Some(MyEvent::MoveDown),
            _ => None
        }
    }
}

