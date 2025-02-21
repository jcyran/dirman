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

use super::{help::HelpWindow, user_input::UserInput};

//Styles
const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(0x3f, 0x44, 0x50));

pub struct App {
    dir: FileManager,
    file_list: FileList,
    select_list: SelectList,
    user_input: UserInput,
    bookmarked: Bookmarked,
    app_mode: AppMode,
    error_msg: String,
}

struct FileList {
    items: Vec<String>,
    state: ListState,
}

struct SelectList {
    items: Vec<String>,
    state: ListState,
}

struct Bookmarked {
    full_path: String,
    file_name: String,
}

#[derive(Debug, EnumIter)]
enum FileAction {
    Delete,
    Rename,
    Bookmark,
}

#[derive(PartialEq, PartialOrd)]
pub enum AppMode {
    Exit,
    Files,
    Select,
    Rename,
    Delete,
    Create,
    Help,
}

impl Default for Bookmarked {
    fn default() -> Self {
        Self {
            full_path: String::default(),
            file_name: String::default(),
        }
    }
}

impl fmt::Display for FileAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for FileAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Delete" => Ok(FileAction::Delete),
            "Rename" => Ok(FileAction::Rename),
            "Bookmark" => Ok(FileAction::Bookmark),
            _ => Err(()),
        }
    }
}

impl Default for SelectList {
    fn default() -> Self {
        SelectList {
            items: FileAction::iter().map(|action| action.to_string()).collect(),
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
            user_input: UserInput::default(),
            bookmarked: Bookmarked::default(),
            app_mode: AppMode::Files,
            error_msg: String::default(),
        }
    }
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.file_list.state.select(Some(0));

        while self.app_mode != AppMode::Exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;

            self.file_list.items = match self.dir.dir_contents() {
                Ok(contents) => contents,
                Err(_) => vec!["No such directory".to_string()],
            };
        }
        
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }

        Ok(())
    }

    // Handling key press events

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.app_mode = AppMode::Exit,
            KeyCode::Char('?') => self.app_mode = AppMode::Help,
            code => {
                match self.app_mode {
                    AppMode::Files => {
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => self.select_previous_file(),
                            KeyCode::Down | KeyCode::Char('j') => self.select_next_file(),
                            KeyCode::Char('a') => self.app_mode = AppMode::Create,
                            KeyCode::Char('m') => self.move_into(),
                            KeyCode::Char('-') => self.move_out(),
                            KeyCode::Char('b') => self.move_bookmarked(),
                            KeyCode::Enter => self.enter_select_menu(),
                            _ => {}
                        }
                    },
                    AppMode::Select => {
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => self.select_previous_action(),
                            KeyCode::Down | KeyCode::Char('j') => self.select_next_action(),
                            KeyCode::Enter => self.select_menu(),
                            KeyCode::Esc => self.exit_select_menu(),
                            _ => {}
                        }
                    },
                    AppMode::Rename => {
                        match code {
                            KeyCode::Enter => {
                                self.rename_file();
                                self.app_mode = AppMode::Files;
                            },
                            KeyCode::Char(to_insert) => self.user_input.enter_char(to_insert),
                            KeyCode::Backspace => self.user_input.delete_char(),
                            KeyCode::Esc => self.app_mode = AppMode::Select,
                            _ => {}
                        }
                    },
                    AppMode::Delete => {
                        match code {
                            KeyCode::Enter => {
                                if self.user_input.get_input_value() == "y" {
                                    self.delete_file()
                                }

                                self.app_mode = AppMode::Files;
                            },
                            KeyCode::Char(to_insert) => self.user_input.enter_char(to_insert),
                            KeyCode::Backspace => self.user_input.delete_char(),
                            KeyCode::Esc => self.app_mode = AppMode::Select,
                            _ => {}
                        }
                    }
                    AppMode::Help => {
                        match code {
                            KeyCode::Esc => self.app_mode = AppMode::Files,
                            _ => {},
                        }
                    },
                    AppMode::Create => {
                        match code {
                            KeyCode::Enter => {
                                self.create_file();
                                self.app_mode = AppMode::Files;
                            },
                            KeyCode::Char(to_insert) => self.user_input.enter_char(to_insert),
                            KeyCode::Backspace => self.user_input.delete_char(),
                            KeyCode::Esc => self.app_mode = AppMode::Select,
                            _ => {}
                        }
                    },
                    AppMode::Exit => {},
                }
            }
        }
    }

    fn select_previous_file(&mut self) {
        self.file_list.state.select_previous();
    }

    fn select_next_file(&mut self) {
        self.file_list.state.select_next();
    }

    fn select_previous_action(&mut self) {
        self.select_list.state.select_previous();
    }

    fn select_next_action(&mut self) {
        self.select_list.state.select_next();
    }

    fn move_into(&mut self) {
        match self.file_list.state.selected() {
            Some(i) => {
                let folder = self.file_list.items[i].to_string();
                self.dir.next_path(folder);

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
        self.app_mode = AppMode::Select;
        self.select_list.state.select(Some(0));
    }

    fn exit_select_menu(&mut self) {
        self.app_mode = AppMode::Files;
        self.select_list.state.select(None);
    }

    fn select_menu(&mut self) {
        let Some(index) = self.file_list.state.selected() else { return };
        let file_name = self.file_list.items[index].clone();

        let Some(index) = self.select_list.state.selected() else { return };
        let Ok(action) = self.select_list.items[index].parse::<FileAction>() else { return };

        match action {
            FileAction::Delete => {
                self.user_input = UserInput::default();
                self.app_mode = AppMode::Delete;
            }
            FileAction::Rename => { 
                self.user_input = UserInput::new(file_name);
                self.app_mode = AppMode::Rename;
            },
            FileAction::Bookmark => {
                let file_path = match self.dir.get_file_path(file_name.clone()) {
                    Ok(path) => path,
                    Err(e) => {
                        self.error_msg = e.to_string();
                        return;
                    }
                };

                self.bookmarked.full_path = file_path;
                self.bookmarked.file_name = file_name;

                self.app_mode = AppMode::Files;
            }
        }
    }

    fn delete_file(&mut self) {
        let Some(index) = self.file_list.state.selected() else { return };
        let file_name = self.file_list.items[index].clone();
        let file_path = match self.dir.get_file_path(file_name.clone()) {
            Ok(path) => path,
            Err(e) => {
                self.error_msg = e.to_string();
                return;
            }
        };

        let metadata = match self.dir.get_metadata(file_name) {
            Some(metadata) => metadata,
            None => return
        };

        match self.dir.delete(file_path, metadata.filetype) {
            Ok(_) => {},
            Err(e) => self.error_msg = e.to_string(),
        };
    }

    fn rename_file(&mut self) {
        let Some(index) = self.file_list.state.selected() else { return };
        let file_name = self.file_list.items[index].clone();
        let file_path = match self.dir.get_file_path(file_name) {
            Ok(path) => path,
            Err(e) => {
                self.error_msg = e.to_string();
                return;
            }
        };

        let new_file_path = match self.dir.get_file_path(self.user_input.get_input_value()) {
            Ok(path) => path,
            Err(e) => {
                self.error_msg = e.to_string();
                return;
            }
        };

        match self.dir.rename(file_path, new_file_path) {
            Ok(_) => {},
            Err(e) => {
                self.error_msg = e.to_string();
            }
        };
    }

    fn move_bookmarked(&mut self) {
        let new_path = match self.dir.get_file_path(self.bookmarked.file_name.clone()) {
            Ok(path) => path,
            Err(e) => {
                self.error_msg = e.to_string();
                return;
            }
        };

        match self.dir.rename(self.bookmarked.full_path.clone(), new_path) {
            Ok(_) => {},
            Err(e) => {
                self.error_msg = e.to_string();
            }
        };
        
        self.bookmarked = Bookmarked::default();
    }

    fn create_file(&mut self) {
        let file_path = match self.dir.get_file_path(self.user_input.get_input_value()) {
            Ok(path) => path,
            Err(e) => {
                self.error_msg = e.to_string();
                return;
            }
        };

        match self.dir.create(file_path) {
            Ok(_) => {},
            Err(e) => {
                self.error_msg = e.to_string();
            }
        };
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_height = area.height.saturating_sub(4);

        let [header_area, mut main_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(main_height),
        ]).areas(area);

        if !self.error_msg.is_empty() {
            let error_area: Rect;

            [error_area, main_area] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1)
            ]).areas(main_area);

            self.render_error(error_area, buf);
        }

        match self.app_mode {
            AppMode::Rename | AppMode::Delete | AppMode::Create => {
                let input_area: Rect;

                [main_area, input_area] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ]).areas(main_area);

                self.render_input(input_area, buf);
            },
            _ => {},
        };

        let [files_area, mut metadata_area] = Layout::horizontal(
            [Constraint::Fill(2), Constraint::Fill(1)]
        ).areas(main_area);

        App::render_header(header_area, buf);
        self.render_files(files_area, buf);

        if !self.bookmarked.file_name.is_empty() {
            let bookmark_area: Rect;

            [metadata_area, bookmark_area] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(2),
            ]).areas(metadata_area);

            self.render_bookmark(bookmark_area, buf);
        }

        match self.app_mode {
            AppMode::Select | AppMode::Rename | AppMode::Delete => {
                let [metadata_area, select_area] = Layout::vertical(
                    [Constraint::Fill(1); 2]
                ).areas(metadata_area);

                self.render_metadata(metadata_area, buf);
                self.render_select_menu(select_area, buf);
            },
            _ => self.render_metadata(metadata_area, buf),
        };

        if self.app_mode == AppMode::Help {
            let help_area = Rect {
                x: area.width / 3,
                y: area.height / 4,
                width: area.width / 3,
                height: area.height / 2,
            };

            HelpWindow::default().render_help(help_area, buf);
        }
    }
}


// Rendering logic
impl App {
    fn render_error(&self, area: Rect, buf: &mut Buffer) {
        let error = Line::from(
            self.error_msg.clone().red()
        );

        Paragraph::new(error)
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Directory Manager")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_files(&mut self, area: Rect, buf: &mut Buffer) {
        let current_path = Line::from(format!(" {} ", self.dir.get_current_path())).left_aligned();

        let instruction = Line::from(vec![
            " Help ".into(),
            "<?> ".blue().into(),
        ]);

        let block = Block::bordered()
            .title(Line::from(" Files "))
            .title_bottom(current_path.yellow())
            .title_bottom(instruction.right_aligned())
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
                " Type: ".blue().into(),
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

    fn render_bookmark(&self, area: Rect, buf: &mut Buffer) {
        let bookmark = Line::from(vec![
            " Bookmark: ".blue().into(),
            self.bookmarked.file_name.clone().into(),
        ]);

        Paragraph::new(bookmark)
            .bold()
            .left_aligned()
            .render(area, buf);
    }

    fn render_select_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let title_bottom = Line::from(vec![
            " Exit ".into(),
            "<Esc> ".blue().into(),
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

    fn render_input(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Input "))
            .border_set(border::THICK);

        let input_text = match self.app_mode {
            AppMode::Rename => Line::from(vec![" Renaming a file: ".blue().into()]),
            AppMode::Delete => {
                let Some(index) = self.file_list.state.selected() else { return };
                let file_name = self.file_list.items[index].clone();

                Line::from(vec![
                    " Delete a file: ".blue().into(),
                    file_name.into(),
                    " (y/n) ".blue().into(),
                ])
            },
            AppMode::Create => Line::from(vec![" Creating a file: ".blue().into()]),
            _ => Line::from(vec!["".into()]),
        };

        let input_value = self.user_input.get_input_value();

        let mut input_block = input_text;
        input_block.spans.push(input_value.into());
        input_block.spans.push("_".yellow().into());

        Paragraph::new(input_block)
            .block(block)
            .bold()
            .left_aligned()
            .render(area, buf);
    }
}

