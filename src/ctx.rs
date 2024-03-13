use crate::runtime;

use super::*;
use futures::{Future, FutureExt};
use minimo::choice;
use std::collections::HashMap;
use std::fmt::Result;
use std::io::Write;
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

use minimo::*;

lazy_static::lazy_static! {
        pub static ref CTX : Ctx = Ctx::empty();
}

pub fn get_ctx() -> Ctx {
    CTX.clone()
}

pub async fn get_children() -> Vec<Cell> {
    CTX.children.clone().lock().await.clone()
}


pub async fn get_variables() -> Vec<Variable> {
    CTX.variables.clone().lock().await.clone()
}

pub async fn get_runtimes() -> Vec<Runtime> {
    CTX.children
        .clone()
        .lock()
        .await
        .iter()
        .filter_map(|cell| match cell {
            Cell::Runtime(runtime) => Some(runtime.clone()),
            _ => None,
        })
        .collect()
}

pub async fn get_tasks() -> Vec<Task> {
    CTX.children
        .clone()
        .lock()
        .await
        .iter()
        .filter_map(|cell| match cell {
            Cell::Task(task) => Some(task.clone()),
            _ => None,
        })
        .collect()
}

pub async fn get_task(name: impl Into<String>) -> Option<Task> {
    let name = name.into();
    CTX.children
        .clone()
        .lock()
        .await
        .iter()
        .filter_map(|cell| match cell {
            Cell::Task(task) => {
                if task.identifer.matches(&name) {
                    Some(task.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
}

pub async fn get_runtime(name: impl Into<String>) -> Option<Runtime> {
    let name = name.into();

    CTX.children
        .clone()
        .lock()
        .await
        .iter()
        .filter_map(|cell| match cell {
            Cell::Runtime(runtime) => {
                if runtime.identifer.matches(&name) {
                    Some(runtime.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
}

pub async fn get_variable(name: impl Into<String>) -> Atom {
    //go through the list of cells from bottom to top
    let name = name.into();
    for cell in CTX.children.clone().lock().await.iter().rev() {
        match cell {
            Cell::Assignment(variable) => {
                if variable.identifier_is(&name) {
                    return variable.value.clone();
                }
            }
            _ => {}
        }
    }
    Atom::Null
}

pub async fn set(cell: impl Into<Cell>) {
    CTX.children.clone().lock().await.push(cell.into());
}

#[derive(Clone, Debug)]
pub struct Ctx {
    pub variables: Arc<Mutex<Vec<Variable>>>,
    pub children: Arc<Mutex<Vec<Cell>>>,
}

impl Ctx {
    pub async fn display_options(&self) {
        let mut choices = vec![];
        for task in get_tasks().await {
            let task = task.clone();
            choices.push(AsyncChoice::new(
                task.name(),
                format!("run {}", task.name().to_lowercase()),
                Arc::new(move || {
                    let task = task.clone();
                    Pin::from(Box::new(async move {
                        runtime::execute(task.get_code(), task.runtime(), "run".to_string())
                            .await
                            .unwrap();
                    }))
                }),
            ));
        }

        choices.push(AsyncChoice::new(
            "exit",
            "exit the program",
            Arc::new(move || {
                Pin::from(Box::new(async move {
                    std::process::exit(0);
                }))
            }),
        ));

        let configurations = get_configurations();
        let choice = display_selection_menu("select a task to run it", &choices, &configurations);
        if let Some(choice) = choice {
            choice.run().await;
        }
    }

    pub fn empty() -> Self {
        Ctx {
            variables: Arc::new(Mutex::new(vec![])),
            children: Arc::new(Mutex::new(vec![])),
        }
    }
}

pub fn get_configurations() -> Vec<AsyncChoice> {
    vec![
        AsyncChoice::new(
            "settings",
            "change settings",
            Arc::new(move || {
                Pin::from(Box::new(async move {
                    // let settings = get_settings().await;
                    // let choice = display_selection_menu("select a setting to change", &settings, &[]);
                    // if let Some(choice) = choice {
                    //     choice.run().await;
                    // }
                }))
            }),
        ),
        AsyncChoice::new(
            "repositories",
            "manage repositories",
            Arc::new(move || {
                Pin::from(Box::new(async move {
                    // let repositories = get_repositories().await;
                    // let choice = display_selection_menu("select a repository to manage", &repositories, &[]);
                    // if let Some(choice) = choice {
                    //     choice.run().await;
                    // }
                }))
            }),
        ),
    ]
}



pub async fn scan() -> std::io::Result<()> {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let pattern = format!("{}/**/*.moto", current_dir.display());
    for entry in glob::glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let content = fs::read_to_string(path).await?;
                let script = ast::parse(&content);
                match script {
                    Ok(script) => {
                        for cell in script {
                            set(cell).await;
                        }
                    }
                    Err(e) => eprintln!("Error parsing file: {:?}", e),
                }
            }
            Err(e) => eprintln!("Error reading file: {:?}", e),
        }
    }
    // divider_vibrant();
    Ok(())
}

pub use async_choice::*;
pub mod async_choice {
    use super::*;
    use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEventState};
    use crossterm::execute;
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen, LeaveAlternateScreen
    };
    use futures::Future;
    use std::fmt::Result;
    use std::io::Write;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct AsyncChoice {
        name: String,
        description: String,
        action: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>>,
    }

    impl AsyncChoice {
        pub fn new(
            name: impl Into<String>,
            description: impl Into<String>,
            action: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>>,
        ) -> Self {
            AsyncChoice {
                name: name.into(),
                description: description.into(),
                action,
            }
        }

        pub async fn run(&self) {
            let action = self.action.clone();
            action().await;
        }

        pub fn get_title(&self) -> &str {
            &self.name
        }

        pub fn get_description(&self) -> &str {
            &self.description
        }
    }

    const BANNER: &str = r#"
                                 __  
                ____ ___  ____  / /_____ 
               / __ `__ \/ __ \/ __/ __ \
              / / / / / / /_/ / /_/ /_/ /
             /_/ /_/ /_/\____/\__/\____/ 
"#;

    fn print_banner() {
        println!("{}", vibrant(BANNER));
        divider_vibrant();
    }

    fn print_guidelines() {
        // showln!(gray_dim, "welcome to ", yellow_bold, "moto", gray_dim, " dynamic scripting runtime");
        showln!(gray_dim, "use the ", yellow_bold, "↓ & ↑", gray_dim, " keys to navigate and ", yellow_bold, "enter", gray_dim, " to select an option");
        showln!(gray_dim, "start ", yellow_bold, "typing", gray_dim, " to search for an option. press ", yellow_bold, "esc", gray_dim, " to exit moto.");
        showln!(gray_dim, "access settings & more by typing ", yellow_bold, ":" , gray_dim, " followed by the command");
        // showln!(gray_dim, "press ", yellow_bold, "enter", gray_dim, " to select an option");
        // showln!(gray_dim, "press ", yellow_bold, "esc", gray_dim, " to exit the menu");
        divider_vibrant();
    }



    /// display selection menu
    /// displays a header with question "what do you want to do?"
    /// and a list of choices that the user can select from using the arrow keys and enter
    /// user can also search by typing the name of the choice and the list will be filtered moving the selected item to the top
    /// the search string is also diplayed in the bottom of the menu. backspace can be used to delete the last character
    /// user can exit the menu by pressing the escape key.

    pub fn display_selection_menu(
        header: &str,
        choices: &[AsyncChoice],
        configurations: &[AsyncChoice],
    ) -> Option<AsyncChoice> {
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(
            std::io::stdout(),
            crossterm::cursor::Hide,
        ).expect("Failed to clear terminal");
    
        print_banner();
        print_guidelines();
    
        let mut selected = 0usize;
        let mut search_text = String::new();
        let mut filtered_choices = choices.to_vec();
        let mut is_config_mode = false;
    
        print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
        loop {
            if let Event::Key(key_event) = event::read().expect("Failed to read event") {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    KeyCode::Char('c')
                        if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                    {
                        print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
                        break;
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                            print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
                        }
                    }
                    KeyCode::Down => {
                        if selected < filtered_choices.len() - 1 {
                            selected += 1;
                            print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
                        }
                    }
                    KeyCode::Enter => {
                        execute!(
                            std::io::stdout(),
                            crossterm::cursor::Show,
                            crossterm::terminal::Clear(ClearType::FromCursorDown),
                        ).expect("Failed to clear terminal");
                        let title = filtered_choices[selected].get_title();
                        showln!(yellow_bold, "╭─ ", gray_dim, "running ", yellow_bold, title, yellow_bold, " ─",yellow_bold,"─".repeat(47 - title.len()));
                        showln!(yellow_bold, "↓");
                        disable_raw_mode().expect("Failed to disable raw mode");
                        return Some(filtered_choices[selected].clone());
                    }
                    KeyCode::Char(':') => {
                        is_config_mode = true;
                        search_text.clear();
                        filtered_choices = configurations.to_vec();
                        selected = 0;
                        print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
                    }
                    KeyCode::Char(c) => {
                        search_text.push(c);
                        filtered_choices = if is_config_mode {
                            filter_choices(configurations, &search_text)
                        } else {
                            filter_choices(choices, &search_text)
                        };
                        selected = 0;
                        print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
                    }
                    KeyCode::Backspace => {
                        search_text.pop();
                        if search_text.is_empty() && is_config_mode {
                            is_config_mode = false;
                            filtered_choices = choices.to_vec();
                        } else {
                            filtered_choices = if is_config_mode {
                                filter_choices(configurations, &search_text)
                            } else {
                                filter_choices(choices, &search_text)
                            };
                        }
                        selected = 0;
                        print_menu(header, &filtered_choices, selected, &search_text, is_config_mode);
                    }
                    KeyCode::Esc => {
                        //clear everything below the menu
                        execute!(
                            std::io::stdout(),
                            crossterm::cursor::Show,
                            crossterm::terminal::Clear(ClearType::FromCursorDown),
                        ).expect("Failed to clear terminal");
                        break;
                    }
                    _ => {}
                }
            }
        }
    
        None
    }


    fn print_menu(header: &str, choices: &[AsyncChoice], selected: usize, search_text: &str, is_config_mode: bool) {
        execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(ClearType::FromCursorDown),
        ).expect("Failed to clear terminal");
    
        if is_config_mode {
            showln!(purple_bold, "╭─ ", white_bold, ":configurations", purple_bold, " ─");
        } else {
            showln!(yellow_bold, "╭─ ", white_bold, header, yellow_bold, " ─");
        }
    
        for (i, choice) in choices.iter().enumerate() {
            if i == selected {
                if is_config_mode {
                    showln!(
                        purple_bold,
                        "│",
                        purplebg,
                        format!(" {} ", choice.get_title()),
                        white,
                        " ",
                        purple_bold,
                        choice.get_description()
                    );
                } else {
                    showln!(
                        yellow_bold,
                        "│",
                        yellowbg,
                        format!(" {} ", choice.get_title()),
                        white,
                        " ",
                        yellow_bold,
                        choice.get_description()
                    );
                }
            } else {
                if is_config_mode {
                    showln!(
                        purple_bold,
                        "│",
                        white,
                        format!(" {} ", choice.get_title()),
                        white,
                        " ",
                        gray_dim,
                        choice.get_description()
                    );
                } else {
                    showln!(
                        yellow_bold,
                        "│",
                        white,
                        format!(" {} ", choice.get_title()),
                        white,
                        " ",
                        gray_dim,
                        choice.get_description()
                    );
                }
            }
        }
    
        if search_text.is_empty() {
            if is_config_mode {
                showln!(purple_bold, "╰─→ ", gray_dim, ":search configurations");
            } else {
                showln!(yellow_bold, "╰─→ ", gray_dim, "search");
            }
        } else {
            if is_config_mode {
                showln!(purple_bold, "╰─→ ", purple_bold, search_text);
            } else {
                showln!(yellow_bold, "╰─→ ", yellow_bold, search_text);
            }
        }
    
        let move_to = choices.len() as u16 + 2;
        execute!(
            std::io::stdout(),
            crossterm::cursor::MoveUp(move_to),
        ).expect("Failed to clear terminal");
    }
    
    fn filter_choices(choices: &[AsyncChoice], filter: &str) -> Vec<AsyncChoice> {
        choices
            .iter()
            .filter(|choice| choice.name.to_lowercase().contains(&filter.to_lowercase()))
            .cloned()
            .collect()
    }
}

pub fn get_local_repository_path(name: &str) -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let path = current_dir.join("repositories").join(name);
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

