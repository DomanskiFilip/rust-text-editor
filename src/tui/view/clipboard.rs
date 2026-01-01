// clipboard module responsible for clipboard operations
use super::View;
use crate::tui::caret::{Caret, Position};
use std::io::Error;

pub fn copy_selection(view: &View) -> Result<(), Error> {
    if let Some(ref selection) = view.selection {
        if !selection.is_active() {
            return Ok(());
        }
        
        let (start, end) = selection.get_range();
        let selected_text = extract_text(view, start, end);
        
        // Copy to clipboard
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(selected_text);
        }
    }
    Ok(())
}

pub fn cut_selection(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    // First copy
    copy_selection(view)?;
    
    // Then delete
    delete_selection(view, caret)?;
    
    Ok(())
}

pub fn paste_from_clipboard(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    // Delete selection first if it exists
    if view.selection.is_some() && view.selection.as_ref().unwrap().is_active() {
        delete_selection(view, caret)?;
    }
    
    // Get text from clipboard
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            insert_text_at_cursor(view, caret, &text)?;
        }
    }
    
    Ok(())
}

pub fn delete_selection(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    if let Some(selection) = view.selection.take() {
        if !selection.is_active() {
            view.selection = Some(selection);
            return Ok(());
        }
        
        let (start, end) = selection.get_range();
        
        // Delete the selected text
        delete_range(view, start, end)?;
        
        // Move cursor to start of deleted range
        let (screen_x, screen_y) = super::helpers::text_to_screen_pos(view, start);
        caret.move_to(Position { x: screen_x, y: screen_y })?;
        
        view.render(caret)?;
    }
    
    Ok(())
}

// Helper: Extract text from selection range
fn extract_text(view: &View, start: crate::core::selection::TextPosition, end: crate::core::selection::TextPosition) -> String {
    let mut result = String::new();
    
    if start.line == end.line {
        // Single line selection
        if let Some(line) = view.buffer.lines.get(start.line) {
            let chars: Vec<char> = line.chars().collect();
            let text: String = chars[start.column.min(chars.len())..end.column.min(chars.len())]
                .iter()
                .collect();
            result.push_str(&text);
        }
    } else {
        // Multi-line selection
        for line_idx in start.line..=end.line {
            if let Some(line) = view.buffer.lines.get(line_idx) {
                let chars: Vec<char> = line.chars().collect();
                
                if line_idx == start.line {
                    // First line: from start.column to end
                    let text: String = chars[start.column.min(chars.len())..].iter().collect();
                    result.push_str(&text);
                    result.push('\n');
                } else if line_idx == end.line {
                    // Last line: from beginning to end.column
                    let text: String = chars[..end.column.min(chars.len())].iter().collect();
                    result.push_str(&text);
                } else {
                    // Middle lines: entire line
                    result.push_str(line);
                    result.push('\n');
                }
            }
        }
    }
    
    result
}

// Helper: Delete text in range
fn delete_range(view: &mut View, start: crate::core::selection::TextPosition, end: crate::core::selection::TextPosition) -> Result<(), Error> {
    if start.line == end.line {
        // Single line deletion
        if let Some(line) = view.buffer.lines.get_mut(start.line) {
            let mut chars: Vec<char> = line.chars().collect();
            chars.drain(start.column..end.column.min(chars.len()));
            *line = chars.into_iter().collect();
        }
    } else {
        // Multi-line deletion
        
        // Get the text before selection on first line
        let before_text = if let Some(line) = view.buffer.lines.get(start.line) {
            let chars: Vec<char> = line.chars().collect();
            chars[..start.column.min(chars.len())].iter().collect::<String>()
        } else {
            String::new()
        };
        
        // Get the text after selection on last line
        let after_text = if let Some(line) = view.buffer.lines.get(end.line) {
            let chars: Vec<char> = line.chars().collect();
            chars[end.column.min(chars.len())..].iter().collect::<String>()
        } else {
            String::new()
        };
        
        // Merge before and after text
        let merged_line = format!("{}{}", before_text, after_text);
        
        // Remove all lines in range
        for _ in start.line..=end.line {
            if start.line < view.buffer.lines.len() {
                view.buffer.lines.remove(start.line);
            }
        }
        
        // Insert merged line
        view.buffer.lines.insert(start.line, merged_line);
    }
    
    Ok(())
}

// Helper: Insert text at cursor position
fn insert_text_at_cursor(view: &mut View, caret: &mut Caret, text: &str) -> Result<(), Error> {
    let pos = caret.get_position();
    let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
    
    // Ensure line exists
    while view.buffer.lines.len() <= buffer_line_idx {
        view.buffer.lines.push(String::new());
    }
    
    // Check if text contains newlines
    if text.contains('\n') {
        // Multi-line paste
        let lines: Vec<&str> = text.lines().collect();
        
        if lines.is_empty() {
            return Ok(());
        }
        
        // Split current line at cursor
        let current_line = &view.buffer.lines[buffer_line_idx];
        let before = current_line.chars().take(char_pos).collect::<String>();
        let after = current_line.chars().skip(char_pos).collect::<String>();
        
        // First line: before + first paste line
        view.buffer.lines[buffer_line_idx] = format!("{}{}", before, lines[0]);
        
        // Insert middle lines
        for (i, line) in lines.iter().enumerate().skip(1) {
            if i == lines.len() - 1 {
                // Last line: last paste line + after
                view.buffer.lines.insert(buffer_line_idx + i, format!("{}{}", line, after));
            } else {
                view.buffer.lines.insert(buffer_line_idx + i, line.to_string());
            }
        }
        
        // Move cursor to end of pasted text
        let final_line = buffer_line_idx + lines.len() - 1;
        let final_col = lines.last().unwrap().len() + if lines.len() == 1 { char_pos } else { 0 };
        
        // Handle scrolling if needed
        let visible_line = final_line.saturating_sub(view.scroll_offset);
        let size = crate::tui::terminal::Terminal::get_size()?;
        let visible_rows = (size.height.saturating_sub(Position::HEADER + 1)) as usize;
        
        if visible_line >= visible_rows {
            view.scroll_offset = final_line.saturating_sub(visible_rows - 1);
        }
        
        view.render(caret)?;
        
        let screen_y = Position::HEADER + (final_line - view.scroll_offset) as u16;
        let screen_x = Position::MARGIN + final_col as u16;
        caret.move_to(Position { x: screen_x, y: screen_y })?;
        
    } else {
        // Single line paste
        let line = &mut view.buffer.lines[buffer_line_idx];
        line.insert_str(char_pos, text);
        
        view.render(caret)?;
        
        // Move cursor forward by pasted text length
        let new_x = pos.x + text.len() as u16;
        caret.move_to(Position { x: new_x, y: pos.y })?;
    }
    
    Ok(())
}