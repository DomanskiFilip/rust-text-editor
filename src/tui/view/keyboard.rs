// keyboard logic with edit tracking for undo/redo
use super::View;
use crate::tui::{
    terminal::Terminal, 
    caret::{Caret, Position}
};
use crate::core::edit_history::{Edit, EditOperation};
use std::io::Error;

pub fn type_character(
    view: &mut View,
    character: char,
    caret: &mut Caret,
) -> Result<Option<EditOperation>, Error> {
    // Delete selection first if it exists, then type
    if view.selection.is_some() && view.selection.as_ref().unwrap().is_active() {
        super::clipboard::delete_selection(view, caret)?;
    }
    
    let size = Terminal::get_size()?;
    let position = caret.get_position();
    
    // Don't allow typing in footer
    if position.y >= size.height - 1 {
        return Ok(None);
    }
    
    let buffer_line_idx = (position.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let char_pos = (position.x as usize).saturating_sub(Position::MARGIN as usize);
    
    // Capture state before edit
    let cursor_before = position;
    let scroll_before = view.scroll_offset;

    while view.buffer.lines.len() <= buffer_line_idx {
        view.buffer.lines.push(String::new());
    }

    // If at end of screen width, wrap to next line
    if position.x >= size.width - 1 {
        insert_newline(view, caret)?;
        return type_character(view, character, caret);
    }

    // Insert the character
    let line = &mut view.buffer.lines[buffer_line_idx];
    if char_pos <= line.len() {
        line.insert(char_pos, character);
    } else {
        line.push(character);
    }

    view.render(caret)?;
    
    let new_offset = caret.move_right(view.scroll_offset, view.buffer.lines.len())?;
    view.scroll_offset = new_offset;
    
    // Create edit operation
    let operation = EditOperation {
        edit: Edit::InsertText {
            line: buffer_line_idx,
            column: char_pos,
            text: character.to_string(),
        },
        cursor_before,
        cursor_after: caret.get_position(),
        scroll_before,
        scroll_after: view.scroll_offset,
    };
    
    Ok(Some(operation))
}

pub fn insert_newline(
    view: &mut View,
    caret: &mut Caret,
) -> Result<Option<EditOperation>, Error> {
    // Delete selection first if it exists
    if view.selection.is_some() && view.selection.as_ref().unwrap().is_active() {
        super::clipboard::delete_selection(view, caret)?;
    }
    
    let size = Terminal::get_size()?;
    let position = caret.get_position();
    
    if position.y >= size.height - 1 {
        return Ok(None);
    }
    
    let cursor_before = position;
    let scroll_before = view.scroll_offset;
    
    let buffer_line_idx = (position.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let char_pos = (position.x as usize).saturating_sub(Position::MARGIN as usize);

    while view.buffer.lines.len() <= buffer_line_idx {
        view.buffer.lines.push(String::new());
    }
    
    // Split the current line
    let current_line = &mut view.buffer.lines[buffer_line_idx];
    let remaining_text = if char_pos < current_line.len() {
        current_line.split_off(char_pos)
    } else {
        String::new()
    };
    
    view.buffer.lines.insert(buffer_line_idx + 1, remaining_text.clone());
    
    view.render(caret)?;
    view.scroll_offset = caret.move_down(view.scroll_offset, view.buffer.lines.len())?;
    view.render(caret)?;
    caret.next_line()?;
    
    let operation = EditOperation {
        edit: Edit::InsertLine {
            line: buffer_line_idx,
            remaining_text,
        },
        cursor_before,
        cursor_after: caret.get_position(),
        scroll_before,
        scroll_after: view.scroll_offset,
    };
    
    Ok(Some(operation))
}

pub fn delete_char(
    view: &mut View,
    caret: &mut Caret,
) -> Result<Option<EditOperation>, Error> {
    if view.selection.is_some() && view.selection.as_ref().unwrap().is_active() {
        return super::clipboard::delete_selection(view, caret);
    }
    
    let pos = caret.get_position();
    let cursor_before = pos;
    let scroll_before = view.scroll_offset;
    
    let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
    
    if buffer_line_idx >= view.buffer.lines.len() {
        return Ok(None);
    }
    
    let line_len = view.buffer.lines[buffer_line_idx].len();
    
    if char_pos < line_len {
        // Delete character at cursor
        let deleted_char = view.buffer.lines[buffer_line_idx].remove(char_pos);
        view.render(caret)?;
        caret.move_to(pos)?;
        
        Ok(Some(EditOperation {
            edit: Edit::DeleteText {
                line: buffer_line_idx,
                column: char_pos,
                text: deleted_char.to_string(),
            },
            cursor_before,
            cursor_after: pos,
            scroll_before,
            scroll_after: view.scroll_offset,
        }))
    } else if buffer_line_idx + 1 < view.buffer.lines.len() {
        // At end of line, merge with next line
        let next_line = view.buffer.lines.remove(buffer_line_idx + 1);
        let first_line_end = view.buffer.lines[buffer_line_idx].len();
        view.buffer.lines[buffer_line_idx].push_str(&next_line);
        view.render(caret)?;
        caret.move_to(pos)?;
        
        Ok(Some(EditOperation {
            edit: Edit::JoinLines {
                line: buffer_line_idx,
                first_line_end,
            },
            cursor_before,
            cursor_after: pos,
            scroll_before,
            scroll_after: view.scroll_offset,
        }))
    } else {
        Ok(None)
    }
}

pub fn backspace(
    view: &mut View,
    caret: &mut Caret,
) -> Result<Option<EditOperation>, Error> {
    if view.selection.is_some() && view.selection.as_ref().unwrap().is_active() {
        return super::clipboard::delete_selection(view, caret);
    }
    
    let pos = caret.get_position();
    let cursor_before = pos;
    let scroll_before = view.scroll_offset;
    
    let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
    
    if char_pos > 0 {
        // Delete character before cursor
        if let Some(line) = view.buffer.lines.get_mut(buffer_line_idx) {
            if char_pos <= line.len() {
                let deleted_char = line.remove(char_pos - 1);
                view.render(caret)?;
                let new_offset = caret.move_left(view.scroll_offset)?;
                view.scroll_offset = new_offset;
                
                return Ok(Some(EditOperation {
                    edit: Edit::DeleteText {
                        line: buffer_line_idx,
                        column: char_pos - 1,
                        text: deleted_char.to_string(),
                    },
                    cursor_before,
                    cursor_after: caret.get_position(),
                    scroll_before,
                    scroll_after: view.scroll_offset,
                }));
            }
        }
    } else if buffer_line_idx > 0 {
        // At beginning of line, merge with previous line
        let prev_line_len = view.buffer.lines[buffer_line_idx - 1].len();
        let current_line_content = view.buffer.lines[buffer_line_idx].clone();
        
        view.buffer.lines[buffer_line_idx - 1].push_str(&current_line_content);
        view.buffer.lines.remove(buffer_line_idx);
        
        // Move cursor to end of previous line
        if pos.y > Position::HEADER {
            view.render(caret)?;
            caret.move_to(Position { 
                x: Position::MARGIN + prev_line_len as u16, 
                y: pos.y - 1 
            })?;
        } else if view.scroll_offset > 0 {
            view.scroll_offset -= 1;
            view.render(caret)?;
            caret.move_to(Position { 
                x: Position::MARGIN + prev_line_len as u16, 
                y: Position::HEADER 
            })?;
        }
        
        return Ok(Some(EditOperation {
            edit: Edit::DeleteLine {
                line: buffer_line_idx,
                content: current_line_content,
                prev_line_end_len: prev_line_len,
            },
            cursor_before,
            cursor_after: caret.get_position(),
            scroll_before,
            scroll_after: view.scroll_offset,
        }));
    }
    
    Ok(None)
}