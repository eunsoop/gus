mod edit;

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
                    KeyCode::Esc => {
                        self.should_exit = true;
                        terminal.clear().unwrap();
                        println!("Exiting GUS...");
                        return Ok(());
                    },
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
        let mut conf = edit::ConfigFile::new(".git/config");

        if id == 0 {
            conf.find_line("core", "sshCommand").map(|line| conf.remove_line(line));
            conf.find_line("user", "email").map(|line| conf.remove_line(line));
            conf.find_line("user", "name").map(|line| conf.remove_line(line));
            conf.find_section("user").map(|line| conf.remove_line(line));
        }else {
            let val = &self.credential_value[id - 1];
            val.get("email")
                .map(|email|
                    conf.find_line("user", "email")
                        .map(|pos| conf.update_line(pos, "email", email.as_str().unwrap()))
                        .or_else(|| Option::from(conf.append_line("user", "email", email.as_str().unwrap())))
                ).unwrap_or_else(|| conf.find_line("user", "email").or_else(|| conf.find_line("user", "email")).map(|_| ()));

            val.get("name")
                .map(|name|
                    conf.find_line("user", "name")
                        .map(|pos| conf.update_line(pos, "name", name.as_str().unwrap()))
                        .or_else(|| Option::from(conf.append_line("user", "name", name.as_str().unwrap())))
                ).unwrap_or_else(|| conf.find_line("user", "name").or_else(|| conf.find_line("user", "name")).map(|_| ()));

            val.get("ssh_key")
                .map(|ssh_key|
                    conf.find_line("core", "sshCommand")
                        .map(|pos| conf.update_line(pos, "sshCommand", format!("\"ssh -i {}\"", ssh_key.as_str().unwrap()).as_str()))
                        .or_else(|| Option::from(conf.append_line("core", "sshCommand", format!("\"ssh -i {}\"", ssh_key.as_str().unwrap()).as_str())))
                ).unwrap_or_else(|| conf.find_line("core", "sshCommand").or_else(|| conf.find_line("core", "sshCommand")).map(|_| ()));
        }

        conf.save(".git/config")
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