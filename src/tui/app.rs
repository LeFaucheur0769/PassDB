use ratatui::DefaultTerminal;
use crate::tui::screen::{Action, Screen};
use crate::tui::context::AppContext;
use crate::tui::screens::main_menu::MainMenuScreen;

pub struct App {
    stack: Vec<Box<dyn Screen>>,
    ctx: AppContext,
    running: bool,
}

impl App {
    pub fn new(ctx: AppContext) -> Self {
        Self {
            stack: vec![Box::new(MainMenuScreen::new())],
            ctx,
            running: true,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        use std::time::Duration;

        while self.running && !self.stack.is_empty() {
            // Draw current screen
            if let Some(screen) = self.stack.last_mut() {
                terminal.draw(|frame| screen.draw(frame))?;
            }

            // Poll for input with a timeout so we keep redrawing even with no keypresses
            if crossterm::event::poll(Duration::from_millis(33))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if let Some(screen) = self.stack.last_mut() {
                        if let Some(action) = screen.handle_key(key, &mut self.ctx) {
                            match action {
                                Action::Push(new_screen) => {
                                    self.stack.push(new_screen);
                                }
                                Action::Pop => {
                                    self.stack.pop();
                                }
                                Action::Replace(new_screen) => {
                                    self.stack.pop();
                                    self.stack.push(new_screen);
                                }
                                Action::Quit => {
                                    self.running = false;
                                    ratatui::restore();
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}