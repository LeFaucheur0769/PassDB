// use crate::logging;
use crate::test_utils;

use std::{
    env::{self},
    fs, io,
    path::{Path, PathBuf},
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    self, Terminal,
    layout::{Constraint, Layout},
    prelude::Backend,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

// #[test]
pub fn test_fun() -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();
    let mut test_app = TestApp::new();
    let mut test_sorting = TestApp::new();
    let result = test_app.test_tui(&mut terminal)?;
    match result {
        "test_sorting" => {
            ratatui::restore();
            test_utils::test_sorting();
        }
        "detecting_if_line_is_ulp" => {
            ratatui::restore();
            use text_io::read;
            println!("Line to sort : ");
            let line: String = read!("{}\n");
            let result = test_sorting.detecting_if_line_is_ulp(line);
            println!("result : {}\n", result);
        }
        "get_current_dir" => {
            ratatui::restore();
            println!("current dir is : {}", test_utils::test_write::get_current_dir())
        },
        "check_for_config_folders" => {
            ratatui::restore();
            test_utils::test_write::check_for_config_folders();
        }
        other => {
            eprintln!("Unknown option: {}", other)
        }
    }
    ratatui::restore();
    println!("Result : {}", result);
    Ok(())
}

fn get_default_import_dir() -> PathBuf {
    let exe_dir = env::current_exe()
        .expect("Failed to get exe path")
        .parent()
        .expect("Failed to get exe directory")
        .to_path_buf();
    exe_dir.join("import")
}
struct TestApp {
    debug: bool,
    db_location: String,
    create_db_in_tool_folder: bool,
    export_results_location: String,
    import_location: String,
    print_result_export_file: bool,
    check_if_valid_combolist: bool,
    nbr_of_check_per_file: i64,
    file_to_sort_location: String,
    fiel_to_sort_not_txt_files: String,
    file_urlloginpass_dir: String,
    add_file: bool,
    logs: Vec<String>,
}

impl TestApp {
    // Just the new fn used to simulate the main app and test things
    pub fn new() -> Self {
        let config_location = "passdb.yml";
        let read_config =
            fs::read_to_string(&config_location).expect("Failed to read the config location");
        let config = yaml_rust2::YamlLoader::load_from_str(&read_config).expect("Invalid YAML");

        // Get either the config path or default, as PathBuf
        let import_path = config[0]["import_location"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| get_default_import_dir());

        // Convert to absolute path
        let import_location = import_path.canonicalize().unwrap_or(import_path); // fallback if folder doesn't exist yet
        TestApp {
            debug: config[0]["debug"].as_bool().unwrap(),
            db_location: config[0]["db_location"].as_str().unwrap().to_string(),
            create_db_in_tool_folder: config[0]["create_db_in_toolFolder"]
                .as_bool()
                .unwrap_or(true),
            export_results_location: config[0]["export_results_location"]
                .as_str()
                .unwrap_or("export/")
                .to_string(),
            import_location: import_location.to_string_lossy().to_string(), // full path
            print_result_export_file: config[0]["print_result_export_file"]
                .as_bool()
                .unwrap_or(false),
            check_if_valid_combolist: config[0]["check_if_valid_combolist"]
                .as_bool()
                .unwrap_or(true),
            nbr_of_check_per_file: config[0]["nbr_of_check_per_file"].as_i64().unwrap_or(10),
            file_to_sort_location: config[0]["file_to_sort_location"]
                .as_str()
                .unwrap_or("to_sort/")
                .to_string(),
            fiel_to_sort_not_txt_files: config[0]["fiel_to_sort_not_txt_files"]
                .as_str()
                .unwrap_or("to_sort/not_txt/")
                .to_string(),
            file_urlloginpass_dir: config[0]["file_urlloginpass_dir"]
                .as_str()
                .unwrap_or("to_sort/url_login_pass/")
                .to_string(),
            add_file: config[0]["add_file"].as_bool().unwrap_or(false),
            logs: vec![],
        }
    }

    // Just the test tui
    pub fn test_tui<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<&str> {
        let menu_test = [
            "test_sorting",
            "detecting_if_line_is_ulp",
            "test_searching",
            "get_current_dir",
            "check_for_config_folders",
            "check",
        ];
        let mut state = ListState::default();
        state.select(Some(0));
        loop {
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let chunks = Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Min(0)])
                        .split(area);
                    let items: Vec<ListItem> = menu_test
                        .iter()
                        .map(|m| ListItem::new(m.to_string()))
                        .collect();
                    let list = List::new(items)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("PassDB test menu"),
                        )
                        .highlight_style(Style::default().fg(Color::Yellow))
                        .highlight_symbol(">>");
                    frame.render_stateful_widget(list, chunks[0], &mut state);
                })
                .map_err(|e| io::Error::other(format!("{e}")))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    KeyCode::Down => {
                        let i = match state.selected() {
                            Some(i) if i + 1 < menu_test.len() => i + 1,
                            _ => 0, // wrap back to top
                        };
                        state.select(Some(i));
                    }

                    KeyCode::Up => {
                        let i = match state.selected() {
                            Some(i) if i > 0 => i - 1,
                            _ => menu_test.len() - 1, // wrap to bottom
                        };
                        state.select(Some(i));
                    }

                    KeyCode::Enter => {
                        if let Some(i) = state.selected() {
                            let selected = menu_test[i];
                            if selected == "Exit" {
                                break;
                            } else {
                                return Ok(selected);
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
        Ok("exit")
    }

    fn ensure_dir(&mut self, path: &str) -> Result<bool, String> {
        if Path::new(path).is_dir() {
            self.logs.push(format!("Directory {} exists", path));
            Ok(true)
        } else {
            self.logs
                .push(format!("Directory {} missing, creating...", path));
            std::fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create {}: {}", path, e))?;
            Ok(true)
        }
    }

    fn detecting_if_line_is_ulp(&mut self, line: String) -> String {
        let result = line;
        return result;
    }
    // fn check_for_valid_app_dir() {
    //     let db_dir =
    // }

    //     fn check_if_valid_or_create_dir(&mut self) -> Result<bool, String> {
    //         let paths = [
    //             // self.db_location.clone(),
    //             // self.export_results_location.clone(),
    //             // self.import_location.clone(),
    //             // self.file_to_sort_location.clone(),
    //         ];

    //         for path in paths.iter() {
    //             self.ensure_dir(path)?;
    //         }

    //         Ok(true)
    // }
}
