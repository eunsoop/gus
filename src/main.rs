use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget};
use ratatui::{DefaultTerminal, Frame, TerminalOptions, Viewport};
use std::{env, fs};
use std::io::stdin;
use toml_edit::{DocumentMut, Item};

fn main() {
    println!("Git User Switcher (GUS)");
    println!("=========================");
    let path = env::home_dir().unwrap().to_str().unwrap().to_owned()+"/.gus/config";

    if !fs::exists(&path).expect("Error checking file existence") {
        fs::write(&path, "").expect("Error creating config file");
    }

    let content = fs::read_to_string(path).unwrap();
    let doc = content.parse::<DocumentMut>().unwrap();

    let terminal = ratatui::init_with_options(TerminalOptions {
        viewport: Viewport::Inline(8)
    });

    let mut list = doc.as_table().iter().map(|(k, _)| String::from(k)).collect::<Vec<String>>();
    list.insert(0, String::from("Global"));
    list.insert(0, String::from("Select a profile to use:"));
    list.push(String::from("Create new"));

    let vlist = doc.as_table().iter().map(|(_, v)| Item::from(v)).collect::<Vec<Item>>();
    let app_result = App::new(list, vlist, doc).run(terminal);

}

struct App {
    should_exit: bool,
    state: ListState,
    credential_list: Vec<String>,
    credential_value: Vec<Item>,
    credentials: DocumentMut,
}

impl App {
    fn new(list: Vec<String>, credential_value: Vec<Item>, credentials: DocumentMut) -> Self {
        Self {
            should_exit: false,
            state: ListState::default(),
            credential_list: list,
            credential_value,
            credentials,
        }
    }

    fn cursor_up(&mut self) {
        self.state.select_previous()
    }

    fn cursor_down(&mut self) {
        self.state.select_next()
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => self.should_exit = true,
                    KeyCode::Up => {
                        if self.state.selected().unwrap_or(0) > 1 {
                            self.cursor_up()
                        }

                    },
                    KeyCode::Down => {
                        if self.state.selected().unwrap_or(0) == 0 {
                            self.state.select(Some(1))
                        }else {
                            self.cursor_down()
                        }
                    },
                    KeyCode::Enter => {
                        let i = self.state.selected().unwrap_or(0);

                        if i == 0 {
                            continue;
                        }

                        if i < self.credential_list.len() - 1 {
                            terminal.clear().unwrap();
                            println!();

                            if !fs::exists(".git/config").expect("Permission denied") {
                                println!(".git/config does not exist!");
                                self.should_exit = true;
                                return Ok(());
                            }

                            if self.set_credential(i-1).is_ok() {
                                println!("Success to change git credential to {}!", &self.credential_list[i]);
                            }
                        } else {
                            self.should_exit = true;
                            terminal.flush().unwrap();
                            println!("Please enter your credential information:");

                            todo!();

                            println!("Profile Name: ");
                            let mut pname = String::new();
                            stdin().read_line(&mut pname).unwrap();
                            pname = pname.trim().to_string();

                            if self.credentials.contains_key(&pname) {
                                println!("Profile name already exists!");
                                return Ok(());
                            }else if pname.is_empty() {
                                println!("Profile name cannot be empty!");
                                return Ok(());
                            }

                            println!("Name: ");
                            let mut name = String::new();
                            stdin().read_line(&mut name).unwrap();

                            if name.trim().is_empty() {
                                println!("Name cannot be empty!");
                                self.should_exit = true;
                                return Ok(());
                            }

                            println!("Email: ");
                            let mut email = String::new();
                            stdin().read_line(&mut email).unwrap();

                            if email.trim().is_empty() {
                                println!("Email cannot be empty!");
                                self.should_exit = true;
                                return Ok(());
                            }

                            println!("SSH Key Path: ");
                            let mut ssh_key = String::new();
                            stdin().read_line(&mut ssh_key).unwrap();
                            ssh_key = ssh_key.trim().to_string();
                            if !fs::exists(&ssh_key).unwrap() {
                                println!("SSH Key does not exist!");
                                self.should_exit = true;
                                return Ok(());
                            }

                            self.credentials[&pname]["email"] = toml_edit::value(email.trim());
                            self.credentials[&pname]["name"] = toml_edit::value(name.trim());
                            self.credentials[&pname]["ssh_key"] = toml_edit::value(ssh_key.trim());

                            fs::write(&ssh_key, self.credentials.to_string()).unwrap();
                            println!("Success to create new credentials profile: {}", pname);
                        }

                        self.should_exit = true;
                    }
                    _ => {}
                }
            }
        }
        return Ok(());
    }

    fn set_credential(&mut self, id: usize) -> Result<(), std::io::Error> {
        let content = fs::read_to_string(".git/config")?;
        let mut doc = content.parse::<DocumentMut>().unwrap();

        if id == 0 {
            if let Some(table) = doc["user"].as_table_mut() {
                table.remove("email");
                table.remove("name");

                if table.is_empty() {
                    doc.remove("user");
                }
            }

            if let Some(table) = doc["core"].as_table_mut() {
                table.remove("sshCommand");
            }
        }else {
            let val = &self.credential_value[id-1];
            doc["user"] = toml_edit::table();
            doc["user"]["email"] = toml_edit::value(val["email"].as_str().unwrap());
            doc["user"]["name"] = toml_edit::value(val["name"].as_str().unwrap());
            doc["core"]["sshCommand"] = toml_edit::value("ssh -i ".to_owned() + val["ssh_key"].as_str().unwrap());
        }

        fs::write(".git/config", doc.to_string())
            .map_err(|e| <std::io::Error as Into<_>>::into(e.into()))
    }

    fn draw(&mut self, frame: &mut Frame) {
        let items: Vec<ListItem> = self
            .credential_list
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i != 0 && i < self.credential_list.len()-1 {
                    ListItem::from(Span::from(i.to_string()+". "+item))
                }else {
                    ListItem::from(Span::from(item))
                }

            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::WhenSelected);

        StatefulWidget::render(list, frame.area(), frame.buffer_mut(), &mut self.state);
    }
}