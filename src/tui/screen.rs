use ratatui::Frame;
use crossterm::event::KeyEvent;
use crate::tui::context::AppContext;

pub trait Screen {
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Option<Action>;
    fn draw(&mut self, frame: &mut Frame);
}

pub enum Action {
    Push(Box<dyn Screen>),
    Pop,
    Replace(Box<dyn Screen>),
    Quit,
}