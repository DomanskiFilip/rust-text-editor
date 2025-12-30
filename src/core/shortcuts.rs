// shortcuts module to handle key mappings
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
            (KeyCode::Char('g'), KeyModifiers::CONTROL, Action::ToggleCtrlShortcuts, "Toggle ctrl shortcuts footer"),
            (KeyCode::Char('q'), KeyModifiers::CONTROL, Action::Quit, "Quit"),
            (KeyCode::Char('s'), KeyModifiers::CONTROL, Action::Save, "Save"),
            (KeyCode::Char('n'), KeyModifiers::CONTROL, Action::New, "New"),
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
    
    // Returns Ctrl shortcuts for footer display
    pub fn get_ctrl_shortcuts() -> Vec<(String, &'static str)> {
        Self::get_mappings()
            .into_iter()
            .filter(|(_, mods, _, _)| mods.contains(KeyModifiers::CONTROL))
            .map(|(code, mods, _, desc)| {
                (Self::key_to_short_string(code, mods), desc)
            })
            .collect()
    }
    
    // print all shortcuts
    pub fn print_all() {        
        for (code, mods, _, desc) in Self::get_mappings() {
            println!("  {:<15} : {}", Self::key_to_string(code, mods), desc);
        }
    }
    
    // used to display shortcuts with flag --shortcuts
    fn key_to_string(code: KeyCode, mods: KeyModifiers) -> String {
        let mut string = String::new();
        if mods.contains(KeyModifiers::CONTROL) { string.push_str("Ctrl+"); }
        if mods.contains(KeyModifiers::ALT) { string.push_str("Alt+"); }
        if mods.contains(KeyModifiers::SHIFT) { string.push_str("Shift+"); }
        match code {
            KeyCode::Char(character) => string.push(character.to_ascii_uppercase()),
            KeyCode::Left => string.push_str("Left Arrow"),
            KeyCode::Right => string.push_str("Right Arrow"),
            KeyCode::Up => string.push_str("Up Arrow"),
            KeyCode::Down => string.push_str("Down Arrow"),
            KeyCode::Enter => string.push_str("Enter"),
            KeyCode::Backspace => string.push_str("Backspace"),
            KeyCode::Delete => string.push_str("Delete"),
            KeyCode::Home => string.push_str("Home"),
            KeyCode::End => string.push_str("End"),
            KeyCode::PageUp => string.push_str("Page Up"),
            KeyCode::PageDown => string.push_str("Page Down"),
            _ => string.push_str("Unknown"),
        }
        string
    }
    
    // Short format for footer display (^Q, ^S, etc.)
    fn key_to_short_string(code: KeyCode, mods: KeyModifiers) -> String {
        let mut string = String::new();
        if mods.contains(KeyModifiers::CONTROL) { string.push('^'); }
        if mods.contains(KeyModifiers::ALT) { string.push_str("A-"); }
        if mods.contains(KeyModifiers::SHIFT) { string.push_str("S-"); }
        match code {
            KeyCode::Char(character) => string.push(character.to_ascii_uppercase()),
            _ => string.push_str("?"),
        }
        string
    }
}