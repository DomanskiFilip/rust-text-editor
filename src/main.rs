mod core;
mod tui;

use std::env;
use std::fs;
use tui::{
    TerminalEditor,
    view::Buffer

};
use core::shortcuts::Shortcuts;

fn main() {
    let args: Vec<String> = env::args().collect();
        
    // Check for flags
    if args.iter().any(|arg| arg == "--shortcuts") {
        Shortcuts::print_all();
        return;
    }

    // Decide which buffer to use
    let buffer = if args.len() > 1 {
        let path = &args[1];
        fs::read_to_string(path)
            .map(Buffer::from_string)
            .unwrap_or_else(|_| Buffer::default())
    } else {
        Buffer::default()
    };

    // Start the editor
    let mut editor = TerminalEditor::new(buffer);
    editor.run();
}