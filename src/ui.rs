use std::io;

use crossterm::event::{self, KeyEventKind};
use ratatui::{
    buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame
};

use crate::events::AppEvents;

#[derive(Debug, Default)]
pub struct App {
    exit: bool
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()>{
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match AppEvents::handle_key_event(key_event) {
                    Some(ev) => {

                    },
                    None => {}
                };
            }
            _ => {}
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instruction = Line::from(vec![
            " Move ".into(),
            "<Enter>".blue().into(),
            " Quit ".into(),
            "<q>".blue().into()
        ]);

        let block = Block::new()
            .title_bottom(instruction.right_aligned())
            .border_set(border::THICK);

        let placeholder_text = Text::from("placeholder");

        Paragraph::new(placeholder_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

