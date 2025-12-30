// view module responsible for render and buffer
use crate::tui::{
    caret::{ Position, Caret },
    terminal::Terminal,
};
use std::io::{ stdout, Error };
use crossterm::{
    style::{ Print, Color, SetForegroundColor, ResetColor, SetBackgroundColor, SetAttribute, Attribute },
    queue,
    cursor::MoveTo,
};

pub struct View {
    buffer: Buffer,
    scroll_offset: usize,
    filename: Option<String>,
    show_shortcuts: bool,
}

pub struct Buffer {
    pub lines: Vec<String> 
}

impl View {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            scroll_offset: 0,
            filename: None,
            show_shortcuts: false,
        }
    }
    
    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }
    
    pub fn toggle_ctrl_shortcuts(&mut self) {
        self.show_shortcuts = !self.show_shortcuts;
    }
    
    pub fn render(&self) -> Result<(), Error> {
        let (cur_x, cur_y) = crossterm::cursor::position()?;
        let size = Terminal::get_size()?;
        self.draw_header()?;
    
        // Visible rows = Total height - Header - Footer
        let visible_rows = (size.height.saturating_sub(Position::HEADER + 1)) as usize;
        
        // Find last line with text for dynamic margin
        let last_non_empty_line = self.buffer.lines.iter()
            .rposition(|line| !line.is_empty())
            .unwrap_or(0);
    
        for row in 0..visible_rows {
            let buffer_line_idx = row + self.scroll_offset;
            let terminal_row = row as u16 + Position::HEADER; 
            
            queue!(stdout(), MoveTo(0, terminal_row))?;
            Terminal::clear_rest_of_line()?;
            
            if buffer_line_idx <= last_non_empty_line {
                self.draw_margin_line(terminal_row, buffer_line_idx)?;
            }
            
            if let Some(line) = self.buffer.lines.get(buffer_line_idx) {
                let max_width = (size.width.saturating_sub(Position::MARGIN)) as usize;
                let truncated_line = if line.len() > max_width { &line[..max_width] } else { line };
                Self::print(truncated_line)?;
            }
        }
        self.draw_footer()?;
        
        // Restore cursor position
        queue!(stdout(), MoveTo(cur_x, cur_y))?;
        Ok(())
    }
    
    fn draw_header(&self) -> Result<(), Error> {
        let size = Terminal::get_size()?;
        queue!(
            stdout(),
            MoveTo(0, 0),
            SetForegroundColor(Color::DarkYellow),
            MoveTo(size.width / 2, 0),
            Print(" Quick Notepad ".to_string()),
            ResetColor
        )?;
        Terminal::clear_rest_of_line()?;
        Ok(())
    }

    fn draw_margin_line(&self, row: u16, buffer_line_idx: usize) -> Result<(), Error> {
        queue!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkYellow),
            Print(format!("{:>3} ", buffer_line_idx + 1)),
            ResetColor
        )?;
        Ok(())
    }

    pub fn draw_footer(&self) -> Result<(), Error> {
        let size = Terminal::get_size()?;
        let footer_row = size.height - 1;
        
        // Clear the footer line and set background
        queue!(
            stdout(),
            MoveTo(0, footer_row),
            SetBackgroundColor(Color::Black),
        )?;
        Terminal::clear_rest_of_line()?;
        
        // Move back to start of footer
        queue!(stdout(), MoveTo(0, footer_row))?;
        
        if self.show_shortcuts {
            // Show shortcuts when Ctrl is held
            self.draw_shortcuts_footer()?;
        } else {
            // Show normal footer with file info and stats
            self.draw_info_footer()?;
        }
        
        queue!(stdout(), ResetColor)?;
        Ok(())
    }
    
    fn draw_info_footer(&self) -> Result<(), Error> {
        use crossterm::cursor::position;
        
        let size = Terminal::get_size()?;
        let footer_row = size.height - 1;
        let (cur_x, cur_y) = position()?;
        
        // Left side: Filename or [No Name]
        queue!(stdout(), MoveTo(1, footer_row))?;
        let filename_display = self.filename.as_deref().unwrap_or("[No Name]");
        queue!(
            stdout(),
            SetBackgroundColor(Color::Black),
            SetForegroundColor(Color::Yellow),
            SetAttribute(Attribute::Bold),
            Print(format!(" {} ", filename_display)),
            SetAttribute(Attribute::Reset),
        )?;
        
        // Calculate stats - find last non-empty line for accurate count
        let total_lines = self.buffer.lines.iter()
            .rposition(|line| !line.is_empty())
            .map(|idx| idx + 1)
            .unwrap_or(1);
        let total_chars: usize = self.buffer.lines.iter()
            .take(total_lines)
            .map(|line| line.len())
            .sum();
        
        // Current position (adjust for margin)
        let line_num = cur_y.saturating_sub(Position::HEADER) + 1 + self.scroll_offset as u16;
        let col_num = cur_x.saturating_sub(Position::MARGIN - 1);
        
        // Middle-left: Stats
        let stats = format!(" Ln {}, Col {} ", line_num, col_num);
        queue!(
            stdout(),
            SetForegroundColor(Color::White),
            Print(stats),
        )?;
        
        // Middle: Lines and Characters count
        let counts = format!("Lines: {} | Chars: {} ", total_lines, total_chars);
        let counts_width = counts.len() as u16;
        let middle_pos = (size.width / 2).saturating_sub(counts_width / 2);
        queue!(
            stdout(),
            MoveTo(middle_pos, footer_row),
            SetBackgroundColor(Color::Black),
            SetForegroundColor(Color::White),
            Print(counts),
        )?;
        
        // Right side: Help hint
        let hint = " Press Ctrl + g for shortcuts ";
        let hint_width = hint.len() as u16;
        let hint_pos = size.width.saturating_sub(hint_width + 1);
        queue!(
            stdout(),
            MoveTo(hint_pos, footer_row),
            SetForegroundColor(Color::DarkYellow),
            SetAttribute(Attribute::Italic),
            Print(hint),
            SetAttribute(Attribute::Reset),
        )?;
        
        Ok(())
    }
    
    fn draw_shortcuts_footer(&self) -> Result<(), Error> {
        use crate::core::shortcuts::Shortcuts;
        
        let size = Terminal::get_size()?;
        let footer_row = size.height - 1;
        
        queue!(
            stdout(),
            MoveTo(1, footer_row),
            SetBackgroundColor(Color::Black),
        )?;
        
        // Get shortcuts from Shortcuts module
        let shortcuts = Shortcuts::get_ctrl_shortcuts();
        
        let mut current_x = 1;
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            // Check if we have space
            let entry_width = key.len() + desc.len() + 4;
            if current_x + entry_width as u16 > size.width - 2 {
                break;
            }
            
            // Draw key in bold yellow
            queue!(
                stdout(),
                MoveTo(current_x, footer_row),
                SetForegroundColor(Color::DarkYellow),
                SetAttribute(Attribute::Bold),
                Print(format!("{}", key)),
                SetAttribute(Attribute::Reset),
            )?;
            current_x += key.len() as u16;
            
            // Draw description
            queue!(
                stdout(),
                MoveTo(current_x, footer_row),
                SetForegroundColor(Color::White),
                Print(format!(" {} ", desc)),
            )?;
            current_x += desc.len() as u16 + 1;
            
            // Add separator except for last item
            if i < shortcuts.len() - 1 {
                queue!(
                    stdout(),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│ "),
                )?;
                current_x += 2;
            }
        }
        
        Ok(())
    }

    pub fn print(text: &str) -> Result<(), Error> {
        queue!(stdout(), Print(text))?;
        Ok(())
    }
    
    pub fn type_character(&mut self, character: char, caret: &mut Caret) -> Result<(), Error> {
        let size = Terminal::get_size()?;
        let position = caret.get_position();
        
        // Don't allow typing in footer
        if position.y >= size.height - 1 {
            return Ok(());
        }
        
        // Adjust Y coordinate to Buffer Index
        let buffer_line_idx = (position.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;

        while self.buffer.lines.len() <= buffer_line_idx {
            self.buffer.lines.push(String::new());
        }

        // Adjust X coordinate to Character Index
        let char_pos = (position.x as usize).saturating_sub(Position::MARGIN as usize);
        
        // If at end of screen width, wrap to next line
        if position.x >= size.width - 1 {
            self.insert_newline(caret)?;
            return self.type_character(character, caret);
        }

        let line = &mut self.buffer.lines[buffer_line_idx];
        if char_pos <= line.len() {
            line.insert(char_pos, character);
        } else {
            line.push(character);
        }

        self.render()?;
        
        // Use caret's move_right to handle cursor movement
        let new_offset = caret.move_right(self.scroll_offset, self.buffer.lines.len())?;
        self.scroll_offset = new_offset;
        
        Ok(())
    }
    
    pub fn insert_newline(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let size = Terminal::get_size()?;
        let position = caret.get_position();
        
        // Prevent typing in the absolute last row of the terminal (footer)
        if position.y >= size.height - 1 {
            return Ok(());
        }
        
        let buffer_line_idx = (position.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        let char_pos = (position.x as usize).saturating_sub(Position::MARGIN as usize);
    
        // Ensure buffer is large enough
        while self.buffer.lines.len() <= buffer_line_idx {
            self.buffer.lines.push(String::new());
        }
        
        // Split the current line
        let current_line = &mut self.buffer.lines[buffer_line_idx];
        let new_line_content = if char_pos < current_line.len() {
            current_line.split_off(char_pos)
        } else {
            String::new()
        };
        
        // Insert the new line into the buffer
        self.buffer.lines.insert(buffer_line_idx + 1, new_line_content);
        
        // Update the view (must render before moving caret so terminal state matches)
        self.render()?;
        
        // This will handle incrementing scroll_offset if the cursor is at the bottom of the visible area
        self.scroll_offset = caret.move_down(self.scroll_offset, self.buffer.lines.len())?;
        
        // Final render to place cursor correctly
        self.render()?;
        caret.next_line()?;
        Ok(())
    }
    
    pub fn move_up(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let new_offset = caret.move_up(self.scroll_offset)?;
        self.scroll_offset = new_offset;
        self.render()?;
        self.clamp_cursor_to_line(caret)?;
        Ok(())
    }
    
    pub fn move_down(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let new_offset = caret.move_down(self.scroll_offset, self.buffer.lines.len())?;
        self.scroll_offset = new_offset;
        self.render()?;
        self.clamp_cursor_to_line(caret)?;
        Ok(())
    }
    
    fn clamp_cursor_to_line(&self, caret: &mut Caret) -> Result<(), Error> {
        let pos = caret.get_position();
        let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        
        if let Some(line) = self.buffer.lines.get(buffer_line_idx) {
            let line_end = Position::MARGIN + line.len() as u16;
            let size = Terminal::get_size()?;
            let max_x = line_end.min(size.width - 1);
            
            if pos.x > max_x {
                caret.move_to(Position { x: max_x, y: pos.y })?;
            } else {
                caret.move_to(pos)?;
            }
        } else {
            caret.move_to(pos)?;
        }
        
        Ok(())
    }
    
    pub fn move_right(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let pos = caret.get_position();
        let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        
        // Check if we can move right on current line
        if let Some(line) = self.buffer.lines.get(buffer_line_idx) {
            let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
            let line_end = Position::MARGIN + line.len() as u16;
            let size = Terminal::get_size()?;
            
            // If we're within the line content, use caret's move_right
            if pos.x < line_end && pos.x < size.width - 1 {
                let new_offset = caret.move_right(self.scroll_offset, self.buffer.lines.len())?;
                self.scroll_offset = new_offset;
                return Ok(());
            }
            
            // If at end of line content, try to move to next line
            if char_pos >= line.len() && buffer_line_idx + 1 < self.buffer.lines.len() {
                if pos.y < size.height - 2 {
                    // Move to start of next line
                    caret.move_to(Position { x: Position::MARGIN, y: pos.y + 1 })?;
                } else {
                    // Scroll down and stay at same y position
                    self.scroll_offset += 1;
                    self.render()?;
                    caret.move_to(Position { x: Position::MARGIN, y: pos.y })?;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn move_left(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let pos = caret.get_position();
        let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        
        // If at beginning of line content, try to move to end of previous line
        if pos.x <= Position::MARGIN && buffer_line_idx > 0 {
            let prev_line_len = self.buffer.lines.get(buffer_line_idx - 1)
                .map(|l| l.len())
                .unwrap_or(0);
            
            if pos.y > Position::HEADER {
                // Move to end of previous line
                caret.move_to(Position { 
                    x: Position::MARGIN + prev_line_len as u16, 
                    y: pos.y - 1 
                })?;
            } else if self.scroll_offset > 0 {
                // Scroll up
                self.scroll_offset -= 1;
                self.render()?;
                caret.move_to(Position { 
                    x: Position::MARGIN + prev_line_len as u16, 
                    y: Position::HEADER 
                })?;
            }
        } else {
            // Normal left movement using caret
            let new_offset = caret.move_left(self.scroll_offset)?;
            self.scroll_offset = new_offset;
        }
        
        Ok(())
    }
    
    pub fn move_top(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let new_offset = caret.move_top()?;
        self.scroll_offset = new_offset;
        self.render()?;
        caret.move_to(Position { x: Position::MARGIN, y: Position::HEADER })?;
        Ok(())
    }
    
    pub fn move_bottom(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let size = Terminal::get_size()?;
        let visible_rows = (size.height.saturating_sub(Position::HEADER + 1)) as usize;
        
        // Find last non-empty line
        let last_line = self.buffer.lines.iter()
            .rposition(|line| !line.is_empty())
            .unwrap_or(0);
        
        // Calculate scroll offset to show last line
        if last_line >= visible_rows {
            self.scroll_offset = last_line - visible_rows + 1;
        } else {
            self.scroll_offset = 0;
        }
        
        self.render()?;
        
        // Use caret's move_bottom to position at bottom of visible area
        caret.move_bottom()?;
        
        // Then clamp to actual line length
        self.clamp_cursor_to_line(caret)?;
        Ok(())
    }
    
    pub fn move_max_left(&mut self, caret: &mut Caret) -> Result<(), Error> {
        caret.move_max_left()?;
        Ok(())
    }
    
    pub fn move_max_right(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let pos = caret.get_position();
        let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        
        // Get the actual line length to determine proper max right position
        if let Some(line) = self.buffer.lines.get(buffer_line_idx) {
            let size = Terminal::get_size()?;
            let line_end = Position::MARGIN + line.len() as u16;
            let max_x = line_end.min(size.width - 1);
            caret.move_to(Position { x: max_x, y: pos.y })?;
        } else {
            // If no line content, just use caret's move_max_right
            caret.move_max_right()?;
        }
        Ok(())
    }
    
    pub fn handle_resize(&mut self, caret: &mut Caret) -> Result<(), Error> {
        caret.clamp_to_bounds()?;
        self.render()?;
        caret.move_to(caret.get_position())?;
        Ok(())
    }
    
    pub fn delete_char(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let pos = caret.get_position();
        let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
        
        // Check if we're in a valid line
        if buffer_line_idx >= self.buffer.lines.len() {
            return Ok(());
        }
        
        let line_len = self.buffer.lines[buffer_line_idx].len();
        
        if char_pos < line_len {
            // Delete character at cursor
            self.buffer.lines[buffer_line_idx].remove(char_pos);
            self.render()?;
            caret.move_to(pos)?;
        } else if buffer_line_idx + 1 < self.buffer.lines.len() {
            // At end of line, merge with next line
            let next_line = self.buffer.lines.remove(buffer_line_idx + 1);
            self.buffer.lines[buffer_line_idx].push_str(&next_line);
            self.render()?;
            caret.move_to(pos)?;
        }
        
        Ok(())
    }
    
    pub fn backspace(&mut self, caret: &mut Caret) -> Result<(), Error> {
        let pos = caret.get_position();
        let buffer_line_idx = (pos.y.saturating_sub(Position::HEADER)) as usize + self.scroll_offset;
        let char_pos = (pos.x as usize).saturating_sub(Position::MARGIN as usize);
        
        if char_pos > 0 {
            // Delete character before cursor
            if let Some(line) = self.buffer.lines.get_mut(buffer_line_idx) {
                if char_pos <= line.len() {
                    line.remove(char_pos - 1);
                    self.render()?;
                    // Use caret's move_left
                    let new_offset = caret.move_left(self.scroll_offset)?;
                    self.scroll_offset = new_offset;
                }
            }
        } else if buffer_line_idx > 0 {
            // At beginning of line, merge with previous line
            let prev_line_len = self.buffer.lines[buffer_line_idx - 1].len();
            let current_line_content = self.buffer.lines[buffer_line_idx].clone();
            
            self.buffer.lines[buffer_line_idx - 1].push_str(&current_line_content);
            self.buffer.lines.remove(buffer_line_idx);
            
            // Move cursor to end of previous line
            if pos.y > Position::HEADER {
                self.render()?;
                caret.move_to(Position { 
                    x: Position::MARGIN + prev_line_len as u16, 
                    y: pos.y - 1 
                })?;
            } else if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
                self.render()?;
                caret.move_to(Position { 
                    x: Position::MARGIN + prev_line_len as u16, 
                    y: Position::HEADER 
                })?;
            }
        }
        Ok(())
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            scroll_offset: 0,
            filename: None,
            show_shortcuts: false,
        }
    }
}

impl Buffer {
    // handle loading a file
    pub fn from_string(content: String) -> Self {
        let mut lines: Vec<String> = content
            .lines()
            .map(|line| line.to_string())
            .collect();
        
        // Ensure there is at least one line if the file is empty
        if lines.is_empty() {
            lines.push(String::new());
        }
        
        // Add buffer space for expansion (500 empty lines after content)
        for _ in 0..500 {
            lines.push(String::new());
        }
        
        Self { lines }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        let mut lines = Vec::new();
        // generate 500 lines of Buffer
        for _ in 0..500 {
            lines.push(String::new());
        }
        Self { lines }
    }
}