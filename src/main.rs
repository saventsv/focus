use std::{default, io::{self, LineWriter}, ops::Deref};
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, terminal};
use ratatui::{
    DefaultTerminal, Frame, layout::Constraint, style::{Stylize, Color}, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}
};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
enum App_State {
    List,
    Todos,
    Reminders,
    Bookmarks,
}

#[derive(Serialize, Deserialize)]
struct App {
    #[serde(skip)]
    cursor: usize,
    input: String,
    adding: bool,
    #[serde(skip)]
    exit: bool,
    reminders: Vec<String>,
    todos: Vec<String>,
    urls: Vec<String>,
    list_options: Vec<String>,
    state: App_State,
}

impl App {

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            terminal.draw(|terminal| self.draw(terminal));
            self.handle_input();
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        
        match self.state {

            App_State::Todos => {
                let screen = ratatui::layout::Layout::default()
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(frame.area());

                let todo_items: Vec<Line> = self.todos.iter()
                    .enumerate()
                    .map(|(i, todo)| {
                        if i == self.cursor {
                            Line::from(vec![
                                " > ".fg(Color::Rgb(163, 190, 140)).into(),
                                todo.as_str().fg(Color::Rgb(163, 190, 140)).bold().underlined().into(),
                            ])
                        } else {
                            Line::from(format!("  {}", todo))
                        }
                    })
                    .collect();

                let input_widget = Paragraph::new(self.input.as_str())
                    .block(Block::bordered().title(" New Todo ").fg(Color::Rgb(94, 129, 172))).fg(Color::Rgb(94, 129, 172)).white();

                let list_widget = Paragraph::new(Text::from(todo_items))
                    .block(Block::bordered().title(" Todos ").fg(Color::Rgb(180, 142, 173)));

                frame.render_widget(input_widget, screen[0]);
                frame.render_widget(list_widget, screen[1]);

                if self.adding {
                    frame.set_cursor_position((
                        screen[0].x + 1 + self.input.chars().count() as u16,
                        screen[0].y + 1,
                    ));
                }

            }
            App_State::Reminders => {
                let screen = ratatui::layout::Layout::default()
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(frame.area());

                let reminder_items: Vec<Line> = self.reminders.iter()
                    .enumerate()
                    .map(|(i, reminder)| {
                        if i == self.cursor {
                            Line::from(vec![
                                " > ".fg(Color::Rgb(163, 190, 140)).into(),
                                reminder.as_str().fg(Color::Rgb(163, 190, 140)).bold().underlined().into(),
                            ])
                        } else {
                            Line::from(format!("  {}", reminder))
                        }
                    })
                    .collect();

                let input_widget = Paragraph::new(self.input.as_str())
                    .block(Block::bordered().title(" New Reminder ").fg(Color::Rgb(94, 129, 172))).white();

                let list_widget = Paragraph::new(Text::from(reminder_items))
                    .block(Block::bordered().title(" Reminders ").fg(Color::Rgb(180, 142, 173)));

                frame.render_widget(input_widget, screen[0]);
                frame.render_widget(list_widget, screen[1]);

                if self.adding {
                    frame.set_cursor_position((
                        screen[0].x + 1 + self.input.chars().count() as u16,
                        screen[0].y + 1,
                    ));
                }

            }
            App_State::Bookmarks => {
                let screen = ratatui::layout::Layout::default()
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(frame.area());

                let bookmarks: Vec<Line> = self.urls.iter()
                    .enumerate()
                    .map(|(i, url)| {
                        if i == self.cursor {
                            Line::from(vec![
                                " > ".fg(Color::Rgb(163, 190, 140)).into(),
                                url.as_str().fg(Color::Rgb(163, 190, 140)).bold().underlined().into(),
                            ])
                        } else {
                            Line::from(format!("  {}", url))
                        }
                    })
                    .collect();

                let input_widget = Paragraph::new(self.input.as_str()) 
                    .block(Block::bordered().title(" New Bookmark ").fg(Color::Rgb(94, 129, 172)));
                let list_widget = Paragraph::new(Text::from(bookmarks))
                    .block(Block::bordered().title(" Bookmarks ").fg(Color::Rgb(180, 142, 173))).fg(Color::Rgb(94, 129, 172)).white();

                frame.render_widget(input_widget, screen[0]);
                frame.render_widget(list_widget, screen[1]);

                if self.adding {
                    frame.set_cursor_position((
                        screen[0].x + 1 + self.input.chars().count() as u16,
                        screen[0].y + 1,
                    ));
                }

            }

            App_State::List => {
                let screen = ratatui::layout::Layout::default()
                    .constraints([Constraint::Min(0)])
                    .split(frame.area());

                let list_options : Vec<Line> = self.list_options.iter()
                    .enumerate()
                    .map(|(i, choice)| {

                        if i == self.cursor {
                            Line::from(vec![
                                " > ".fg(Color::Rgb(163, 190, 140)).into(),
                                choice.as_str().fg(Color::Rgb(163, 190, 140)).bold().underlined().into(),
                            ])
                        } else {
                            Line::from(format!("  {}", choice))
                        }
                    })
                    .collect();

                let list_widget = Paragraph::new(Text::from(list_options))
                    .block(Block::bordered().title(" Home ").fg(Color::Rgb(180, 142, 173))).white();


                frame.render_widget(list_widget, screen[0]);
            }
        }
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key)
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match (&self.state, self.adding, key.code, ){
            (_ , true, KeyCode::Enter) => self.submit(),
            (_, true, KeyCode::Esc) => self.adding = false,
            (_, true, KeyCode::Char(c)) => self.input_push(c),
            (_, true, KeyCode::Backspace) => { self.input.pop(); },

            (App_State::List, false, KeyCode::Char('j')) => self.move_cursor_down(),
            (App_State::List, false, KeyCode::Char('k')) => self.move_cursor_up(),
            (App_State::List, false, KeyCode::Enter) => self.confirm(),
            (App_State::List, false, KeyCode::Char('l')) => self.confirm(),


            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('a')) => self.adding = true,
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('i')) => self.adding = true,
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('j')) => self.move_cursor_down(),
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('k')) => self.move_cursor_up(),
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Esc) => self.state = App_State::List,
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('h')) => self.state = App_State::List,
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('d')) => self.remove_element(),
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('J')) => self.move_element_down(),
            (App_State::Todos | App_State::Reminders | App_State::Bookmarks, false, KeyCode::Char('K')) => self.move_element_up(),

            (App_State::Bookmarks, false, KeyCode::Enter) => self.open_url(),

            (_, false, KeyCode::Char('q')) => self.exit = true,

            _ => {}
        }
    }

    fn submit(&mut self) {
        match self.state {
            App_State::Todos => {
                self.todos.push(self.input.drain(..).collect());
                self.save_to_file();
                self.adding = false;
            }
            App_State::Reminders => {
                self.reminders.push(self.input.drain(..).collect()); 
                self.save_to_file();
                self.adding = false  
            }
            App_State::Bookmarks => {
                self.urls.push(self.input.drain(..).collect()); 
                self.save_to_file();
                self.adding = false  
            }


            App_State::List => {}
        }
    }



    fn input_push(&mut self, c: char) {
        self.input.push(c);
    }

    fn move_cursor_up(&mut self) {
        match self.state {
            App_State::List => {
                if self.cursor == 0 {
                    self.cursor = self.list_options.len().saturating_sub(1);
                } else {
                    self.cursor -= 1
                }
            }

            App_State::Todos => {
                if self.cursor == 0 {
                    self.cursor = self.todos.len().saturating_sub(1);
                } else {
                    self.cursor -= 1
                }
            }

            App_State::Reminders => {
                if self.cursor == 0 {
                    self.cursor = self.reminders.len().saturating_sub(1);
                } else {
                    self.cursor -= 1
                }
            }

            App_State::Bookmarks => {
                if self.cursor == 0 {
                    self.cursor = self.urls.len().saturating_sub(1);
                } else {
                    self.cursor -= 1
                }
            }
        }
    }
    
    fn move_cursor_down(&mut self) {
        match self.state {

            App_State::List => {
                if self.cursor == self.list_options.len().saturating_sub(1) {
                    self.cursor = 0
                } else {
                    self.cursor += 1
                }
            }
            App_State::Todos => {
                if self.cursor == self.todos.len().saturating_sub(1) {
                    self.cursor = 0
                } else {
                    self.cursor += 1
                }
            }
            App_State::Reminders => {
                if self.cursor == self.reminders.len().saturating_sub(1) {
                    self.cursor = 0
                } else {
                    self.cursor += 1
                }
            }

            App_State::Bookmarks => {
                if self.cursor == self.urls.len().saturating_sub(1) {
                    self.cursor = 0
                } else {
                    self.cursor += 1
                }
            }

        }
    }

    fn remove_element(&mut self) {
        match self.state {
           
            App_State::Todos => {
                if !self.todos.is_empty() {
                    self.todos.remove(self.cursor);
                    self.save_to_file();

                    if self.cursor >= self.todos.len().saturating_sub(1) && !self.todos.is_empty() {
                        self.cursor = self.todos.len() -1;
                    }
                }
            }

            App_State::Reminders => {
                if !self.reminders.is_empty() {
                    self.reminders.remove(self.cursor);
                    self.save_to_file();

                    if self.cursor >= self.reminders.len().saturating_sub(1) && !self.reminders.is_empty() {
                        self.cursor = self.reminders.len() -1;
                    }
                }
            }

            App_State::Bookmarks => {
                if !self.urls.is_empty() {
                    self.urls.remove(self.cursor);
                    self.save_to_file();

                    if self.cursor >= self.urls.len().saturating_sub(1) && !self.urls.is_empty() {
                        self.cursor = self.urls.len() -1;
                    }
                }
            }

            _ | App_State::List => {}
        }
    }

    fn move_element_down(&mut self) {
        match self.state {
            
            App_State::Todos => {
                if self.cursor != self.todos.len().saturating_sub(1) {
                    self.todos.swap(self.cursor, self.cursor + 1);
                }
            }

            App_State::Reminders => {
                if self.cursor != self.reminders.len().saturating_sub(1) {
                    self.reminders.swap(self.cursor, self.cursor + 1);
                }
            }

            App_State::Bookmarks => {
                if self.cursor != self.urls.len().saturating_sub(1) {
                    self.urls.swap(self.cursor, self.cursor + 1);
                }
            }

            _ | App_State::List => {}
        }
    }

    fn move_element_up(&mut self) {
        match self.state {

            App_State::Todos => {
                if self.cursor != 0 {
                    self.todos.swap(self.cursor, self.cursor -1);
                }
            }

            App_State::Reminders => {
                if self.cursor != 0 {
                    self.reminders.swap(self.cursor, self.cursor -1);
                }
            }

            App_State::Bookmarks => {
                if self.cursor != 0 {
                    self.urls.swap(self.cursor, self.cursor -1);
                }
            }

            _ | App_State::List => {}
        }
    }

    fn confirm(&mut self) {
        match self.cursor {
            0 => self.state = App_State::Todos, 
            1 => self.state = App_State::Reminders, 
            2 => self.state = App_State::Bookmarks, 
            _ => {}
        }
        self.cursor = 0;
    }

    fn open_url(&self) {
        let url = &self.urls[self.cursor];
        if let Err(e) = open::that(url) {
                eprintln!("Failed to open browser {}", e)
        }
    }

    fn get_data_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "yourname", "rust-tui-app") {
            let config_dir = proj_dirs.config_dir();
            let _ = std::fs::create_dir_all(config_dir);
            return config_dir.join("data.json");
        }
        PathBuf::from("data.json")
    }

    fn save_to_file(&self) -> io::Result<()> {
        let path = Self::get_data_path();
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn load_from_file() -> Self {
        let path = Self::get_data_path();
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(app) = serde_json::from_str::<App>(&data) {
                return app;
            }
        }
        App::default()
    }

    
}

impl Default for App {
    
    fn default() -> Self {
        Self {
            cursor: 0,
            input: String::new(),
            adding: false,
            exit: false,
            reminders: vec!["Put Reminders for Yourself Here".to_string()],
            todos: vec!["Put Todos Here".to_string()],
            list_options: vec!["Todo's".to_string(), "Reminders".to_string(), "Bookmarks".to_string()],
            state: App_State::List,
            urls: vec!["https://github.com/".to_string()],
        }
    }
}




fn main() -> io::Result<()> {

    let mut terminal = ratatui::init();
    
    let mut app = App::load_from_file(); 
    
    let app_result = app.run(&mut terminal);
    
    ratatui::restore();

    if let Err(e) = app.save_to_file() {
        eprintln!("Warning: Failed to save data: {}", e);
    }

    app_result;
    Ok(())
}
