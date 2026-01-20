mod core;
mod tui;
mod gui;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check if GUI mode is requested
    let gui_mode = args.iter().any(|arg| arg == "--gui");
    
    // Check for shortcuts flag (works in both modes)
    if args.iter().any(|arg| arg == "--shortcuts") {
        core::shortcuts::Shortcuts::print_all();
        return;
    }
    
    if gui_mode {
        // Launch GUI mode
        let file_path = args.iter()
            .position(|arg| arg != "--gui" && arg != &args[0])
            .map(|i| args[i].clone());
        
        gui::run(file_path);
    } else {
        // Launch TUI mode (existing code)
        let mut editor = if args.len() > 1 {
            let raw_path = &args[1];
            let path_buf = std::fs::canonicalize(raw_path).unwrap_or_else(|_| std::path::PathBuf::from(raw_path));
            
            // Extract full path for the backend
            let full_path = path_buf.to_string_lossy().into_owned();
            
            // Extract just the file name for the display/UI
            let display_name = path_buf
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| full_path.clone());
            
            let extension = path_buf.extension().map(|ext| ext.to_string_lossy().into_owned());
            let friendly_type = core::tabs::get_friendly_filetype(extension);
            
            // Open editor with selected file
            match tui::TerminalEditor::new_with_file(&full_path) {
                Ok(mut ed) => {
                    // We pass 'display_name' to the view so only the name shows in the status bar
                    // while the 'full_path' remains stored in the tab for saving logic
                    ed.set_filename_and_filetype(Some(display_name), friendly_type);
                    ed
                },
                Err(e) => {
                    eprintln!("Error opening file {}: {}", full_path, e);
                    tui::TerminalEditor::new(core::buffer::Buffer::default())
                }
            }
        } else {
            // Default to empty editor if no file specified
            tui::TerminalEditor::new(core::buffer::Buffer::default())
        };
        
        editor.run();
    }
}