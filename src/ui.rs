use core::fmt;
use std::{io, vec};
use strum::IntoEnumIterator;

use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border, text::Line, widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget}, DefaultTerminal
};
use strum_macros::EnumIter;

use crate::directory::FileManager;

//Styles
const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(0x3f, 0x44, 0x50));

pub struct App {
    dir: FileManager,
    file_list: FileList,
    select_list: SelectList,
    select: bool,
    exit: bool,
}

struct FileList {
    items: Vec<String>,
    state: ListState,
}

struct SelectList {
    items: Vec<String>,
    state: ListState,
}

#[derive(Debug, EnumIter)]
enum Actions {
    Delete,
    Rename,
    Move,
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for Actions {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Delete" => Ok(Actions::Delete),
            "Rename" => Ok(Actions::Rename),
            "Move" => Ok(Actions::Move),
            _ => Err(()),
        }
    }
}

impl Default for SelectList {
    fn default() -> Self {
        SelectList {
            items: Actions::iter().map(|action| action.to_string()).collect(),
            state: ListState::default(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let dir = FileManager::default();
        let items = match dir.dir_contents() {
            Ok(contents) => contents,
            Err(_) => vec!["No such directory".to_string()],
        };

        Self {
            dir,
            file_list: FileList { items, state: ListState::default() },
            select_list: SelectList::default(),
            select: false,
            exit: false,
        }
    }
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.file_list.state.select(Some(0));

        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()>{
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('m') => self.move_into(),
            KeyCode::Char('-') => self.move_out(),
            KeyCode::Enter => self.enter_select_menu(),
            KeyCode::Esc => self.exit_select_menu(),
            _ => {}
        }
    }

    fn select_previous(&mut self) {
        if self.select {
            self.select_list.state.select_previous();
        } else {
            self.file_list.state.select_previous();
        }
    }

    fn select_next(&mut self) {
        if self.select {
            self.select_list.state.select_next();
        } else {
            self.file_list.state.select_next();
        }
    }

    fn move_into(&mut self) {
        match self.file_list.state.selected() {
            Some(i) => {
                let folder = self.file_list.items[i].to_string();
                self.dir.next_path(folder);
                self.file_list.items = match self.dir.dir_contents() {
                    Ok(contents) => contents,
                    Err(_) => vec!["No such directory".to_string()],
                };
            }
            None => {}
        };
    }

    fn move_out(&mut self) {
        self.dir.previous_path();
        self.file_list.items = match self.dir.dir_contents() {
            Ok(contents) => contents,
            Err(_) => vec!["No such directory".to_string()],
        };
    }

    fn enter_select_menu(&mut self) {
        if self.select {
            let Some(index) = self.file_list.state.selected() else { return };
            let file_name = self.file_list.items[index].clone();

            let Some(index) = self.select_list.state.selected() else { return };
            let Ok(action) = self.select_list.items[index].parse::<Actions>() else { return };

            match action {
                Actions::Delete => {}
                Actions::Rename => self.dir.rename(file_name),
                Actions::Move => {}
            }
        } else {
            self.select = true;
            self.select_list.state.select(Some(0));
        }
    }

    fn exit_select_menu(&mut self) {
        if self.select {
            self.select = false;
            self.select_list.state.select(None);
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_height = area.height.saturating_sub(4);

        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(main_height),
            Constraint::Length(1),
        ]).areas(area);

        let [files_area, metadata_area] = Layout::horizontal(
            [Constraint::Fill(2), Constraint::Fill(1)]
        ).areas(main_area);


        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_files(files_area, buf);

        if self.select {
            let [metadata_area, select_area] = Layout::vertical(
                [Constraint::Fill(1); 2]
            ).areas(metadata_area);

            self.render_metadata(metadata_area, buf);
            self.render_select_menu(select_area, buf);
        } else {
            self.render_metadata(metadata_area, buf);
        }
    }
}


// Rendering logic
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Directory Manager")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        let instruction = Line::from(vec![
            " Move ".into(),
            "<↓↑>".blue().into(),
            " Move into ".into(),
            "<m>".blue().into(),
            " Move out ".into(),
            "<->".blue().into(),
            " Select ".into(),
            "<Enter>".blue().into(),
            " Quit ".into(),
            "<q>".blue().into(),
            "\t".into()
        ]);

        Paragraph::new(instruction)
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_files(&mut self, area: Rect, buf: &mut Buffer) {
        let current_path = Line::from(format!(" {} ", self.dir.get_current_path())).left_aligned();

        let block = Block::bordered()
            .title(Line::from(" Files "))
            .title_bottom(current_path)
            .border_set(border::THICK);

        let items: Vec<ListItem> = self
            .file_list
            .items
            .iter()
            .map(|item| ListItem::from(format!(" {}", item)))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.file_list.state);
    }

    fn render_metadata(&mut self, area: Rect, buf: &mut Buffer) {
        let file_name = match self.file_list.state.selected() {
            Some(i) => self.file_list.items[i].to_string(),
            None => "".to_string()
        };

        let metadata = match self.dir.get_metadata(file_name) {
            Some(metadata) => metadata,
            None => return
        };

        let block = Block::bordered()
            .title(Line::from(" Properties "))
            .border_set(border::THICK);

        let info = vec![
            Line::from(vec![
                " Filename: ".blue().into(),
                metadata.file_name.into(),
            ]),
            Line::from(vec![
                " Filetype: ".blue().into(),
                metadata.filetype.to_string().into(),
            ]),
            Line::from(vec![
                " Size: ".blue().into(),
                metadata.size.to_string().into(),
                " B".into(),
            ]),
        ];

        Paragraph::new(info)
            .block(block)
            .bold()
            .left_aligned()
            .render(area, buf);
    }

    fn render_select_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let title_bottom = Line::from(vec![
            " Exit ".blue().into(),
            "<Esc> ".into(),
        ]);

        let block = Block::bordered()
            .title(Line::from(" Action "))
            .title_bottom(title_bottom.right_aligned())
            .border_set(border::THICK);

        let items: Vec<ListItem> = self
            .select_list
            .items
            .iter()
            .map(|item| ListItem::from(format!(" {}", item)))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.select_list.state);
    }
}

