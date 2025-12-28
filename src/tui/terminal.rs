// terminal module handles terminal related operations
use crate::tui::drawing::Draw;
use crossterm::{
    cursor::{ MoveTo, position },
    execute,
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode, DisableLineWrap },
};
use std::io::stdout;

pub struct Terminal;

impl Terminal {
    pub fn initialize() {
        match enable_raw_mode() {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to enable raw mode: {error:#?}");
                return;
            }
        }
        
        match execute!(stdout(), DisableLineWrap) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to disable line wrap: {error:#?}");
                return;
            }
        }

        match Self::clear_screen() {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to clear screen: {error:#?}");
                return;
            }
        }

        match Draw::draw_margin() {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to margin: {error:#?}");
                return;
            }
        }

        match Draw::draw_footer() {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to draw footer: {error:#?}");
                return;
            }
        }

        match Self::move_cursor_to(4, 0) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to move cursor: {error:#?}");
                return;
            }
        }
    }

    pub fn terminate() {
        match disable_raw_mode() {
            Ok(_) => {}
            Err(error) => eprintln!("Failed to disable raw mode: {error:#?}"),
        }

        match Self::clear_screen() {
            Ok(_) => {}
            Err(error) => eprintln!("Failed to clear screen: {error:#?}"),
        }

        match Self::move_cursor_to(0, 0) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to move cursor: {error:#?}");
                return;
            }
        }

        println!("Goodbye.");
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        match execute!(stdout(), Clear(ClearType::All)) {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn move_cursor_to(x: u16, y: u16) -> Result<(), std::io::Error> {
        match execute!(stdout(), MoveTo(x, y)) {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn next_line() -> Result<(), std::io::Error> {
        match execute!(stdout(), MoveTo(4, position().unwrap().1 + 1)) {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }
}
