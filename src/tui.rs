// editor module handles the editor's state and logic
mod view;
mod main_error_wrapper;
mod terminal;
mod caret;

use crate::core::actions::Action;
use crate::core::shortcuts::Shortcuts;
use crossterm::{
    event::{ Event, KeyCode, KeyEventKind, read },
    cursor::position,
};
use main_error_wrapper::MainErrorWrapper;
use view::View;
use terminal::Terminal;
use caret::Caret;


pub struct TerminalEditor {
    view: View,
    quit_program: bool,
}

impl TerminalEditor {
    pub fn default() -> Self {
        Self {
            view: View::default(),
            quit_program: false,
        }
    }
    
    pub fn run(&mut self) {
        // initialise tui
        if let Err(error) = Terminal::initialize(&self.view) {
            eprintln!("Terminal Initialisation Failed: {:?}", error); 
        }
        // runs main program loop with error wrapper
        MainErrorWrapper::handle_error(self.main_loop());
        // terminate tui
        if let Err(error) = Terminal::terminate() {
            eprintln!("Terminal Termination Failed: {:?}", error); 
        }
    }

    // main program loop
    fn main_loop(&mut self) -> Result<(), std::io::Error> {
        loop {
            if let Event::Key(event) = read()? {
                if event.kind == KeyEventKind::Press {
                    // Shortcuts resolves key events into actions
                    if let Some(action) = Shortcuts::resolve(&event) {
                        // logic to handle actions
                        match action {
                            Action::Left => Caret::move_left()?,
                            Action::Right => Caret::move_right()?,
                            Action::Up => Caret::move_up()?,
                            Action::Down => Caret::move_down()?,
                            Action::Top => Caret::move_top()?,
                            Action::Bottom => Caret::move_bottom()?,
                            Action::MaxLeft => Caret::move_max_left()?,
                            Action::MaxRight => Caret::move_max_right()?,
                            Action::NextLine => self.view.insert_newline()?,
                            Action::Quit => self.quit_program = true,
                            Action::Print => {
                                if let KeyCode::Char(character) = event.code {
                                    // type/print character into buffer
                                    self.view.type_character(character)?;
                                }
                            }
                        }
                        Terminal::execute()?;
                    }
                }
            }

            if self.quit_program { break; }
        }
        Ok(())
    }
}