// editor module handles the editor's state and logic
pub mod view;
mod main_error_wrapper;
mod terminal;
mod caret;

use crate::core::{
    actions::Action,
    shortcuts::Shortcuts
};
use crossterm::event::{ Event, KeyCode, KeyEventKind, read };
use main_error_wrapper::MainErrorWrapper;
use view::{ View, Buffer };
use terminal::Terminal;
use caret::Caret;

pub struct TerminalEditor {
    view: View,
    caret: Caret,
    quit_program: bool,
}

impl TerminalEditor {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            view: View::new(buffer),
            caret: Caret::new(),
            quit_program: false,
        }
    }
    
    pub fn set_filename(&mut self, filename: String) {
        self.view.set_filename(filename);
    }
    
    pub fn run(&mut self) {
        if let Err(error) = Terminal::initialize(&self.view, &mut self.caret) {
            eprintln!("Terminal Initialisation Failed: {:?}", error); 
        }
        MainErrorWrapper::handle_error(self.main_loop());
        if let Err(error) = Terminal::terminate() {
            eprintln!("Terminal Termination Failed: {:?}", error); 
        }
    }
    
    fn main_loop(&mut self) -> Result<(), std::io::Error> {
        
        loop {
                match read()? {
                    Event::Key(event) => {                        
                        if event.kind == KeyEventKind::Press {
                            if let Some(action) = Shortcuts::resolve(&event) {
                                match action {
                                    Action::Left => self.view.move_left(&mut self.caret)?,
                                    Action::Right => self.view.move_right(&mut self.caret)?,
                                    Action::Up => self.view.move_up(&mut self.caret)?,
                                    Action::Down => self.view.move_down(&mut self.caret)?,
                                    Action::Top => self.view.move_top(&mut self.caret)?,
                                    Action::Bottom => self.view.move_bottom(&mut self.caret)?,
                                    Action::MaxLeft => self.view.move_max_left(&mut self.caret)?,
                                    Action::MaxRight => self.view.move_max_right(&mut self.caret)?,
                                    Action::NextLine => self.view.insert_newline(&mut self.caret)?,
                                    Action::Backspace => self.view.backspace(&mut self.caret)?,
                                    Action::Delete => self.view.delete_char(&mut self.caret)?,
                                    Action::ToggleCtrlShortcuts => {
                                        self.view.toggle_ctrl_shortcuts();
                                        self.view.render()?;
                                    },
                                    Action::Save => {
                                        // TODO: Implement save
                                    },
                                    Action::New => {
                                        // TODO: Implement new file
                                    },
                                    Action::Quit => self.quit_program = true,
                                    Action::Print => {
                                        if let KeyCode::Char(character) = event.code {
                                            self.view.type_character(character, &mut self.caret)?;
                                        }
                                    }
                                }
                                Terminal::execute()?;
                            }
                        }
                    },
                    Event::Resize(_, _) => self.view.handle_resize(&mut self.caret)?,
                    _ => {}
                }
            if self.quit_program { break; }
        }
        Ok(())
    }
}