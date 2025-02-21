use ratatui::{buffer::Buffer, layout::Rect, style::{Style, Stylize}, symbols::border, text::Line, widgets::{Block, Paragraph, Widget}};

pub struct HelpWindow {
    commands: Vec<Command>,
}

struct Command {
    name: String,
    keybind: String,
}

impl Command {
    pub fn get_line(&self) -> Line {
        Line::from(vec![
            self.name.clone().into(),
            self.keybind.clone().blue().into(),
        ])
    }
}

impl Default for HelpWindow {
    fn default() -> Self {
        Self {
            commands: vec![
                Command { name: "Move".to_string(), keybind: "<↓↑>".to_string() },
                Command { name: "Move Into".to_string(), keybind: "<m>".to_string() },
                Command { name: "Move Out".to_string(), keybind: "<->".to_string() },
                Command { name: "Select".to_string(), keybind: "<Enter>".to_string() },
                Command { name: "Move Bookmarked".to_string(), keybind: "<b>".to_string() },
                Command { name: "Quit".to_string(), keybind: "<q>".to_string() },
                Command { name: "Create".to_string(), keybind: "<a>".to_string() },
            ],
        }
    }
}

impl HelpWindow {
    pub fn render_help(&self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default());

        let commands = self.commands
            .iter()
            .map(|c| c.get_line())
            .collect::<Vec<Line>>();

        let bottom_title = Line::from(vec![
            " Close ".into(),
            "<Esc>".blue().into(),
        ]);

        let block = Block::bordered()
            .title(Line::from(" Help "))
            .title_bottom(bottom_title.right_aligned())
            .border_set(border::THICK);

        Paragraph::new(commands)
            .block(block)
            .bold()
            .centered()
            .render(area, buf);
    }
}

