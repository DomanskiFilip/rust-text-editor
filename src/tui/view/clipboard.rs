// clipboard module with EditOperation tracking
use super::View;
use crate::tui::caret::{Caret, Position};
use crate::tui::terminal::Terminal;
use crate::core::selection::TextPosition;
use crate::core::edit_history::{Edit, EditOperation};
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

pub fn cut_selection(view: &mut View, caret: &mut Caret) -> Result<Option<EditOperation>, Error> {
    // First copy
    copy_selection(view)?;
    
    // Then delete and return the operation
    delete_selection(view, caret)
}

pub fn paste_from_clipboard(view: &mut View, caret: &mut Caret) -> Result<Option<EditOperation>, Error> {
    // Delete selection first if it exists
    if view.selection.is_some() && view.selection.as_ref().unwrap().is_active() {
        delete_selection(view, caret)?;
    }
    
    // Get text from clipboard
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            return insert_text_at_cursor(view, caret, &text);
        }
    }
    
    Ok(None)
}

pub fn delete_selection(view: &mut View, caret: &mut Caret) -> Result<Option<EditOperation>, Error> {
    if let Some(selection) = view.selection.take() {
        if !selection.is_active() {
            view.selection = Some(selection);
            return Ok(None);
        }
        
        let (start, end) = selection.get_range();
        let cursor_before = caret.get_position();
        let scroll_before = view.scroll_offset;
        
        // Extract the text that will be deleted (for undo)
        let deleted_text = extract_text(view, start, end);
        
        // Delete the selected text
        delete_range(view, start, end)?;
        
        // Move cursor to start of deleted range
        let (screen_x, screen_y) = super::helpers::text_to_screen_pos(view, start);
        caret.move_to(Position { x: screen_x, y: screen_y })?;
        
        view.render(caret)?;
        
        // Create edit operation
        let operation = EditOperation {
            edit: Edit::ReplaceRange {
                start_line: start.line,
                start_column: start.column,
                end_line: end.line,
                end_column: end.column,
                old_text: deleted_text,
                new_text: String::new(),
            },
            cursor_before,
            cursor_after: caret.get_position(),
            scroll_before,
            scroll_after: view.scroll_offset,
        };
        
        return Ok(Some(operation));
    }
    
    Ok(None)
}

// Helper: Extract text from selection range
fn extract_text(view: &View, start: TextPosition, end: TextPosition) -> String {
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
fn delete_range(view: &mut View, start: TextPosition, end: TextPosition) -> Result<(), Error> {
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

// Helper: Insert text at cursor position and return EditOperation
fn insert_text_at_cursor(view: &mut View, caret: &mut Caret, text: &str) -> Result<Option<EditOperation>, Error> {
    let pos = caret.get_position();
    let cursor_before = pos;
    let scroll_before = view.scroll_offset;
    
    let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
    
    // Ensure line exists
    while view.buffer.lines.len() <= buffer_line_idx {
        view.buffer.lines.push(String::new());
    }
    
    // Check if text contains newlines
    if text.contains('\n') {
        // Multi-line paste
        let lines: Vec<&str> = text.split('\n').collect();
        
        if lines.is_empty() {
            return Ok(None);
        }
        
        // Split current line at cursor
        let current_line = view.buffer.lines[buffer_line_idx].clone();
        let before: String = current_line.chars().take(char_pos).collect();
        let after: String = current_line.chars().skip(char_pos).collect();
        
        // Update first line: before + first pasted line
        view.buffer.lines[buffer_line_idx] = format!("{}{}", before, lines[0]);
        
        // Insert all middle and last lines
        let mut insert_idx = buffer_line_idx + 1;
        for i in 1..lines.len() {
            let line_content = if i == lines.len() - 1 {
                format!("{}{}", lines[i], after)
            } else {
                lines[i].to_string()
            };
            
            if insert_idx < view.buffer.lines.len() {
                view.buffer.lines.insert(insert_idx, line_content);
            } else {
                view.buffer.lines.push(line_content);
            }
            insert_idx += 1;
        }
        
        // Calculate final position
        let final_buffer_line = buffer_line_idx + lines.len() - 1;
        let final_buffer_col = lines[lines.len() - 1].len();
        
        // Update scroll_offset
        let size = Terminal::get_size()?;
        let visible_rows = size.height.saturating_sub(Position::HEADER + 1) as usize;
        
        if final_buffer_line >= view.scroll_offset + visible_rows {
            view.scroll_offset = final_buffer_line.saturating_sub(visible_rows / 2);
        } else if final_buffer_line < view.scroll_offset {
            view.scroll_offset = final_buffer_line;
        }
        
        view.render(caret)?;
        
        let final_text_pos = TextPosition {
            line: final_buffer_line,
            column: final_buffer_col,
        };
        let (screen_x, screen_y) = super::helpers::text_to_screen_pos(view, final_text_pos);
        caret.move_to(Position { x: screen_x, y: screen_y })?;
        
        // Create edit operation for multi-line paste
        Ok(Some(EditOperation {
            edit: Edit::ReplaceRange {
                start_line: buffer_line_idx,
                start_column: char_pos,
                end_line: buffer_line_idx,
                end_column: char_pos,
                old_text: String::new(),
                new_text: text.to_string(),
            },
            cursor_before,
            cursor_after: caret.get_position(),
            scroll_before,
            scroll_after: view.scroll_offset,
        }))
    } else {
        // Single line paste
        let line = &mut view.buffer.lines[buffer_line_idx];
        
        if char_pos <= line.len() {
            line.insert_str(char_pos, text);
        } else {
            line.push_str(&" ".repeat(char_pos - line.len()));
            line.push_str(text);
        }
        
        view.render(caret)?;
        
        let size = Terminal::get_size()?;
        let new_x = (pos.x + text.len() as u16).min(size.width.saturating_sub(1));
        caret.move_to(Position { x: new_x, y: pos.y })?;
        
        // Create edit operation for single-line paste
        Ok(Some(EditOperation {
            edit: Edit::InsertText {
                line: buffer_line_idx,
                column: char_pos,
                text: text.to_string(),
            },
            cursor_before,
            cursor_after: caret.get_position(),
            scroll_before,
            scroll_after: view.scroll_offset,
        }))
    }
}