// shortcuts module handles key events and resolves them into actions
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers };
use crate::core::actions::Action;

pub struct Shortcuts;

impl Shortcuts {
    pub fn resolve(event: &KeyEvent) -> Option<Action> {
        match (event.code, event.modifiers) {
            // caret left
            (KeyCode::Left, _) => Some(Action::Left),
            // caret right
            (KeyCode::Right, _) => Some(Action::Right),
            // caret up
            (KeyCode::Up, _) => Some(Action::Up),
            // caret down
            (KeyCode::Down, _) => Some(Action::Down),
            // caret top line
            (KeyCode::PageUp, _) => Some(Action::Top),
            // caret bottom line
            (KeyCode::PageDown, _) => Some(Action::Bottom),
            // caret most left column
            (KeyCode::Home, _) => Some(Action::MaxLeft),
            // caret most right column
            (KeyCode::End, _) => Some(Action::MaxRight),
            // next line
            (KeyCode::Enter, _) => Some(Action::NextLine),
            // backspace - delete before cursor
            (KeyCode::Backspace, _) => Some(Action::Backspace),
            // delete - delete at cursor
            (KeyCode::Delete, _) => Some(Action::Delete),
            // exit the program
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),
            // print everything else
            _ => Some(Action::Print),
        }
    }
}