// search module for text search functionality
use super::View;
use crate::tui::{
    caret::{Caret, Position},
    terminal::Terminal,
};
use crate::core::selection::{Selection, TextPosition};
use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use std::io::Error;

// Stores all search match locations
#[derive(Clone, Debug)]
pub struct SearchMatch {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

pub struct SearchState {
    pub _query: String,
    pub matches: Vec<SearchMatch>,
    pub current_match_idx: usize,
}

impl SearchState {
    pub fn new(_query: String, matches: Vec<SearchMatch>) -> Self {
        Self {
            _query,
            matches,
            current_match_idx: 0,
        }
    }
    
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match_idx = (self.current_match_idx + 1) % self.matches.len();
        }
    }
    
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match_idx = if self.current_match_idx == 0 {
                self.matches.len() - 1
            } else {
                self.current_match_idx - 1
            };
        }
    }
    
    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.matches.get(self.current_match_idx)
    }
}

pub fn search(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    // Show search prompt
    view.show_prompt(
        super::PromptKind::Search,
        "Search:".to_string(),
    );
    view.needs_redraw = true;
    view.render_if_needed(caret, false)?;
    Terminal::execute()?;

    let mut search_query = String::new();

    // Capture input for search query
    loop {
        match read()? {
            Event::Key(event) if event.kind == KeyEventKind::Press => {
                match event.code {
                    KeyCode::Char(c) => {
                        search_query.push(c);
                        view.append_prompt_char(c);
                        view.render_if_needed(caret, false)?;
                        Terminal::execute()?;
                    }
                    KeyCode::Backspace => {
                        search_query.pop();
                        view.backspace_prompt();
                        view.render_if_needed(caret, false)?;
                        Terminal::execute()?;
                    }
                    KeyCode::Enter => {
                        view.clear_prompt();
                        if !search_query.is_empty() {
                            perform_search(view, caret, &search_query)?;
                        }
                        break;
                    }
                    KeyCode::Esc => {
                        view.clear_prompt();
                        view.render_if_needed(caret, false)?;
                        Terminal::execute()?;
                        break;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn perform_search(view: &mut View, caret: &mut Caret, query: &str) -> Result<(), Error> {
    if query.is_empty() {
        return Ok(());
    }

    // Find all occurrences
    let matches = find_all_occurrences(&view.buffer.lines, query);

    if matches.is_empty() {
        // No match found - show error in prompt
        view.show_prompt(
            super::PromptKind::Error,
            format!("No matches found for '{}'", query),
        );
        view.render_if_needed(caret, false)?;
        Terminal::execute()?;
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        view.clear_prompt();
        view.render_if_needed(caret, false)?;
        Terminal::execute()?;
        return Ok(());
    }

    // Find the match closest to current cursor position
    let current_pos = caret.get_position();
    let current_line = (current_pos.y.saturating_sub(Position::HEADER)) as usize + view.scroll_offset;
    let current_col = (current_pos.x as usize).saturating_sub(Position::MARGIN as usize);
    
    let closest_idx = find_closest_match(&matches, current_line, current_col);

    // Store search state in view
    let search_state = SearchState::new(query.to_string(), matches);
    view.set_search_state(Some(search_state));
    view.set_current_match(closest_idx);

    // Move to first match
    move_to_current_match(view, caret)?;

    Ok(())
}

fn find_all_occurrences(lines: &[String], query: &str) -> Vec<SearchMatch> {
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        let mut start = 0;

        while let Some(pos) = line_lower[start..].find(&query_lower) {
            matches.push(SearchMatch {
                line: line_idx,
                column: start + pos,
                length: query.len(),
            });
            start += pos + 1; // Move past this match to find next
        }
    }

    matches
}

fn find_closest_match(matches: &[SearchMatch], line: usize, col: usize) -> usize {
    let mut closest_idx = 0;
    let mut min_distance = usize::MAX;

    for (idx, m) in matches.iter().enumerate() {
        // Calculate distance (prioritize line, then column)
        let distance = if m.line == line {
            if m.column >= col {
                m.column - col
            } else {
                usize::MAX / 2 + (col - m.column)
            }
        } else if m.line > line {
            (m.line - line) * 1000 + m.column
        } else {
            usize::MAX - (line - m.line) * 1000
        };

        if distance < min_distance {
            min_distance = distance;
            closest_idx = idx;
        }
    }

    closest_idx
}

fn move_to_current_match(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    if let Some(search_state) = &view.search_state {
        if let Some(m) = search_state.current_match() {
            let size = Terminal::get_size()?;
            let visible_rows = size.height.saturating_sub(Position::HEADER + 1) as usize;

            // Adjust scroll to show the match
            if m.line < view.scroll_offset {
                view.scroll_offset = m.line;
            } else if m.line >= view.scroll_offset + visible_rows {
                view.scroll_offset = m.line.saturating_sub(visible_rows / 2);
            }

            // Create selection for current match
            let start_pos = TextPosition {
                line: m.line,
                column: m.column,
            };
            let end_pos = TextPosition {
                line: m.line,
                column: m.column + m.length,
            };

            view.selection = Some(Selection {
                anchor: start_pos,
                cursor: end_pos,
            });

            // Update footer to show match info
            let total = search_state.matches.len();
            let current = search_state.current_match_idx + 1;
            view.show_prompt(
                super::PromptKind::SearchInfo,
                format!("Match {} of {} | ↑/↓ to navigate", current, total),
            );

            // Move caret to end of match
            let (screen_x, screen_y) = super::helpers::text_to_screen_pos(view, end_pos);
            
            view.needs_redraw = true;
            view.render(caret)?;
            caret.move_to(Position { x: screen_x, y: screen_y })?;
            Terminal::execute()?;
        }
    }

    Ok(())
}

pub fn next_search_match(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    if let Some(search_state) = &mut view.search_state {
        search_state.next_match();
        move_to_current_match(view, caret)?;
    }
    Ok(())
}

pub fn prev_search_match(view: &mut View, caret: &mut Caret) -> Result<(), Error> {
    if let Some(search_state) = &mut view.search_state {
        search_state.prev_match();
        move_to_current_match(view, caret)?;
    }
    Ok(())
}

pub fn clear_search(view: &mut View) {
    view.search_state = None;
    view.selection = None;
    view.clear_prompt();
    view.needs_redraw = true;
}