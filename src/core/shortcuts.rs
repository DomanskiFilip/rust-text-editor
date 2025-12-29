// shortcuts.rs
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers };
use crate::core::actions::Action;

pub struct Shortcuts;

impl Shortcuts {
    /// THE ONE PLACE TO CHANGE SHORTCUTS
    /// Format: (KeyCode, Modifiers, Action, Description)
    fn get_mappings() -> Vec<(KeyCode, KeyModifiers, Action, &'static str)> {
        vec![
            (KeyCode::Left, KeyModifiers::empty(), Action::Left, "Move caret left"),
            (KeyCode::Right, KeyModifiers::empty(), Action::Right, "Move caret right"),
            (KeyCode::Up, KeyModifiers::empty(), Action::Up, "Move caret up"),
            (KeyCode::Down, KeyModifiers::empty(), Action::Down, "Move caret down"),
            (KeyCode::PageUp, KeyModifiers::empty(), Action::Top, "Move to top of view"),
            (KeyCode::PageDown, KeyModifiers::empty(), Action::Bottom, "Move to bottom of view"),
            (KeyCode::Home, KeyModifiers::empty(), Action::MaxLeft, "Move to start of line"),
            (KeyCode::End, KeyModifiers::empty(), Action::MaxRight, "Move to end of line"),
            (KeyCode::Enter, KeyModifiers::empty(), Action::NextLine, "Insert new line"),
            (KeyCode::Backspace, KeyModifiers::empty(), Action::Backspace, "Delete before cursor"),
            (KeyCode::Delete, KeyModifiers::empty(), Action::Delete, "Delete at cursor"),
            (KeyCode::Char('q'), KeyModifiers::CONTROL, Action::Quit, "Quit program"),
        ]
    }

    pub fn resolve(event: &KeyEvent) -> Option<Action> {
        // check mappings
        for (code, mods, action, _) in Self::get_mappings() {
            if event.code == code && event.modifiers.contains(mods) {
                return Some(action);
            }
        }

        // Fallback for typing characters
        if let KeyCode::Char(_) = event.code {
            return Some(Action::Print);
        }

        None
    }

    // print all shortcuts
    pub fn print_all() {        
        for (code, mods, _, desc) in Self::get_mappings() {
            println!("  {:<15} : {}", Self::key_to_string(code, mods), desc);
        }
    }

    fn key_to_string(code: KeyCode, mods: KeyModifiers) -> String {
        let mut s = String::new();
        if mods.contains(KeyModifiers::CONTROL) { s.push_str("Ctrl+"); }
        if mods.contains(KeyModifiers::ALT) { s.push_str("Alt+"); }
        if mods.contains(KeyModifiers::SHIFT) { s.push_str("Shift+"); }

        match code {
            KeyCode::Char(c) => s.push(c.to_ascii_uppercase()),
            KeyCode::Left => s.push_str("Left Arrow"),
            KeyCode::Right => s.push_str("Right Arrow"),
            KeyCode::Up => s.push_str("Up Arrow"),
            KeyCode::Down => s.push_str("Down Arrow"),
            KeyCode::Enter => s.push_str("Enter"),
            KeyCode::Backspace => s.push_str("Backspace"),
            KeyCode::Delete => s.push_str("Delete"),
            KeyCode::Home => s.push_str("Home"),
            KeyCode::End => s.push_str("End"),
            KeyCode::PageUp => s.push_str("Page Up"),
            KeyCode::PageDown => s.push_str("Page Down"),
            _ => s.push_str("Unknown"),
        }
        s
    }
}