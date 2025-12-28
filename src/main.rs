mod core;
mod tui;

use tui::TerminalEditor;

fn main() {
    // run the main program loop
    TerminalEditor::default().run();
}
