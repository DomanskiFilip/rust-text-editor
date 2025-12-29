mod core;
mod tui;

use std::env;
use std::fs;
use tui::TerminalEditor;
use tui::view::Buffer;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let path = &args[1];
        let buffer = fs::read_to_string(path)
            .map(Buffer::from_string)
            .unwrap_or_else(|_| Buffer::default());
        
        TerminalEditor::new(buffer).run();
    } else {
        // default buffer
        TerminalEditor::default().run();
    }
}