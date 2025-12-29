// view module responsible for render and buffer
use crate::tui::{
    caret::{ Position, Caret },
    terminal::Terminal,
};
use std::io::{stdout, Error };
use crossterm::{
    style::{Print, Color, SetForegroundColor, ResetColor},
    queue,
    cursor::MoveTo,
};

#[derive(Default)]
pub struct View {
    buffer: Buffer
}

pub struct Buffer {
    pub lines: Vec<String> 
}

impl View {
    
    pub fn render(&self) -> Result<(), Error> {
        let (cur_x, cur_y) = crossterm::cursor::position()?;
        let current_pos = Position { x: cur_x, y: cur_y };
        let size = Terminal::get_size()?;
        
        for row in 0..size.height - 1 {
            // Position cursor at start of row
            queue!(stdout(), MoveTo(0, row))?;

            // Draw Margin
            self.draw_margin_line(row)?;
            
            // Draw Buffer Content
            if let Some(line) = self.buffer.lines.get(row as usize) {
                let max_width = (size.width.saturating_sub(4)) as usize;
                let truncated_line = if line.len() > max_width {
                    &line[..max_width]
                } else {
                    line
                };
                Self::print(truncated_line)?;
            }
        }
        
        // Draw Footer
        self.draw_footer()?;

        // Restore original cursor
        Caret::move_caret_to(current_pos)?;
        Ok(())
    }

    // Helper to draw a single margin line to keep render() clean
    fn draw_margin_line(&self, row: u16) -> Result<(), Error> {
        queue!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("{:>3} ", row + 1)),
            ResetColor
        )?;
        Ok(())
    }

    pub fn draw_footer(&self) -> Result<(), Error> {
        let size = Terminal::get_size()?;
        let footer_row = size.height - 1;
        
        queue!(
            stdout(),
            MoveTo(0, footer_row), 
            SetForegroundColor(Color::DarkGrey),
            Print("ctrl + q = quit |"),
            MoveTo(size.width / 2, footer_row),
            Print("© Filip Domanski"),
            ResetColor,
        )?;
        Ok(())
    }

    pub fn print(text: &str) -> Result<(), Error> {
        queue!(stdout(), Print(text))?;
        Ok(())
    }
    
    pub fn type_character(&mut self, character: char) -> Result<(), Error> {
        let (x, y) = crossterm::cursor::position()?;
        let size = Terminal::get_size()?;
        let y_idx = y as usize;

        // Ensure buffer is long enough
        while self.buffer.lines.len() <= y_idx {
            self.buffer.lines.push(String::new());
        }

        // If at end of screen, wrap
        if x >= size.width - 1 {
            self.insert_newline()?;
            // Re-fetch position after wrap
            return self.type_character(character);
        }

        // Insert character into data
        let char_pos = (x as usize).saturating_sub(4);
        let line = &mut self.buffer.lines[y_idx];
        if char_pos <= line.len() {
            line.insert(char_pos, character);
        } else {
            line.push(character);
        }

        // Move cursor and then render
        self.render()?;
        Caret::move_right()?; 
        Ok(())
    }
    
    pub fn insert_newline(&mut self) -> Result<(), Error> {
        let (x, y) = crossterm::cursor::position()?;
        let y_idx = y as usize;
        let char_pos = (x as usize).saturating_sub(4);
        let size = Terminal::get_size()?;

        // Don't allow newlines on footer
        if y >= size.height - 2 {
            return Ok(());
        }

        // ensure line exists
        while self.buffer.lines.len() <= y_idx {
            self.buffer.lines.push(String::new());
        }
        // Split the data
        let current_line = &mut self.buffer.lines[y_idx];
        let new_line_content = if char_pos < current_line.len() {
            current_line.split_off(char_pos)
        } else {
            String::new()
        };
        // Clear rest of line
        Terminal::clear_rest_of_line()?;
        // insert split into new line
        self.buffer.lines.insert(y_idx + 1, new_line_content);

        // Move cursor to new line position before rendering
        Caret::next_line()?;
        
        self.render()?;
        Ok(())
    }
}

impl Default for Buffer {
     fn default() -> Self{
        let mut lines = Vec::new();
        for _ in 0..500 {
            lines.push(String::new());
        }
        Self { lines }
    }
}