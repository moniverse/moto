use super::*;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEventState};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen, LeaveAlternateScreen
};
use futures::{Future, FutureExt};
use std::collections::HashMap;
use std::env;
use std::fmt::Result;
use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs;

const BANNER: &str = r#"
                                 __  
                ____ ___  ____  / /_____ 
               / __ `__ \/ __ \/ __/ __ \
              / / / / / / /_/ / /_/ /_/ /
             /_/ /_/ /_/\____/\__/\____/ 
"#;

pub fn print_banner() {
    println!("{}", vibrant(BANNER));
    divider_vibrant();
}

fn print_guidelines() {
    showln!(gray_dim, "use the ", yellow_bold, "↓ & ↑", gray_dim, " keys to navigate and ", yellow_bold, "enter", gray_dim, " to select an option");
    showln!(gray_dim, "start ", yellow_bold, "typing", gray_dim, " to search for an option. press ", yellow_bold, "esc", gray_dim, " to exit moto.");
    showln!(gray_dim, "access settings & more by typing ", yellow_bold, ":" , gray_dim, " followed by the command");
    divider_vibrant();
}


#[derive(Clone)]
pub struct AsyncChoice {
    name: String,
    description: String,
    action: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>>,
    file_path: String,
}

impl AsyncChoice {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        action: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>>,
        file_path: impl Into<String>,
    ) -> Self {
        AsyncChoice {
            name: name.into(),
            description: description.into(),
            action,
            file_path: file_path.into(),
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

    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
}

impl PartialEq for AsyncChoice {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.file_path == other.file_path
    }
}



pub async fn scan() -> std::io::Result<()> {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let pattern = format!("{}/*.moto", current_dir.to_str().unwrap());
    for entry in glob::glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let content = fs::read_to_string(path.clone()).await?;
                let script = ast::parse(&content);
                let package_name =  //filename without extension
                    path.file_stem().unwrap_or_default().to_str().unwrap_or_default();

                match script {
                    Ok(script) => {
                       push_cell( Package::new(package_name, script)).await;
            
                    }
                    Err(e) =>  {
                        eprintln!("Error parsing file: {:?}", e);
                        // set(Package::new(package_name, vec![]))
                    }
                }
            }
            Err(e) => eprintln!("Error reading file: {:?}", e),
        }
    }
    Ok(())
}

/// display selection menu
/// displays a header with question "what do you want to do?"
/// and a list of choices that the user can select from using the arrow keys and enter
/// user can also search by typing the name of the choice and the list will be filtered moving the selected item to the top
/// the search string is also diplayed in the bottom of the menu. backspace can be used to delete the last character
/// user can exit the menu by pressing the escape key.


fn filter_choices(choices: &[AsyncChoice], filter: &str) -> Vec<AsyncChoice> {
    choices
        .iter()
        .filter(|choice| choice.name.to_lowercase().contains(&filter.to_lowercase()))
        .cloned()
        .collect()
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
                    // showln!(yellow_bold, "↓");
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
                    //terminate the program
                    std::process::exit(0);
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
                    gray_dim,
                    "|",
                    yellow_bold,
                    choice.get_file_path(),
                    gray_dim,
                    "|",
                    cyan_bold,
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
                    format!(" {}", choice.get_title()),
                    gray_dim,
                    "|",
                    gray_dim,
                    choice.get_file_path()
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

pub async fn display_options() ->  AsyncChoice {
    let  choices = get_tasks().await.into_iter().map(|task| task.into()).collect::<Vec<AsyncChoice>>()
                    .into_iter().chain(default_choices().into_iter()).collect::<Vec<AsyncChoice>>();
    let configurations = get_configurations().await;
    let mut selection = None;
    while selection.is_none() {
        selection = display_selection_menu("what do you want to do?", &choices, &configurations);
    }
    selection.unwrap()
}

/// handling args
/// moto <task_name> will run the task with the name <task_name>.
/// if no task with the name <task_name> is found, the user will be prompted to select a task from the list of available tasks.
/// moto <task_name> [:vname = whatever the content until next occurance of `[:` or eof 
/// this will allow users to provide long sentences as variables without having to use quotes
pub async fn handle_args() -> Option<AsyncChoice> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        showln!(gray_dim, "searching for ", yellow_bold, &args[1], gray_dim, "...");
        let (task_name, variables) = parse_args(&args);
        let mut matched = None;
        
        for task in get_tasks().await {
            if task.name().to_lowercase().trim() == task_name {
                matched = Some(task);
                break;
            }
        }
        for var in variables {
            showln!(cyan_bold, &var.name(), gray_dim, " = ", white, &var.get_value_str());
            set_variable(var.name(), var.get_value()).await;
        }

        divider_vibrant();

        if let Some(task) = matched {
            let title = task.name();
            showln!(yellow_bold, "╭─ ", gray_dim, "running ", yellow_bold, title, yellow_bold, " ─",yellow_bold,"─".repeat(47 - title.len()));
            return Some(task.into());
        } else {
            showln!(orange_bold, "could not find ", gray_dim,"a task with the name ", yellow_bold, &task_name, gray_dim, "... ");
            return None;
        }
    } else {
        None
    }


}


impl From<Task> for AsyncChoice {
    fn from(task: Task) -> Self {
        let name = task.name();
        let description = format!("{}", task.runtime());
        let file_path = env::current_dir().unwrap_or_default().to_str().unwrap_or_default().to_string();

        AsyncChoice::new(name, description,  Arc::new(move || {
            let task = task.clone();
            let code = task.get_code();
            let runtime = task.runtime();
            Pin::from(Box::new(async move {
                execute(code, runtime, "run").await.unwrap();
            }))
        }), file_path)
    }
}



fn parse_args(args: &[String]) -> (String, Vec<Variable>) {
    let mut variables = Vec::new();
    let mut task_name = String::new();

    for arg in args.iter().skip(1) {
        if let Some(start) = arg.find("[:") {
            if let Some(end) = arg[start..].find(']') {
                let var_str = &arg[start + 2..start + end];
                if let Some(eq_pos) = var_str.find('=') {
                    let name = var_str[..eq_pos].to_string();
                    let content = var_str[eq_pos + 1..].to_string();
                    variables.push(Variable::new(name, content));
                }
            }
        } else if task_name.is_empty() { // Assuming the first non-variable argument is the task name
            task_name = arg.to_lowercase(); // Keeping the task name lowercase for consistency
        }
    }

    (task_name, variables)
}


fn default_choices() ->   Vec<AsyncChoice> {
    vec![
        AsyncChoice::new(
            "exit",
            "exit the program",
            Arc::new(move || {
                Pin::from(Box::new(async move {
                    showln!(yellow_bold, "╰─→ ", gray_dim, "exiting moto...");
                    std::process::exit(0);
                }))
            }),
            "".to_string(),
        ),
    ]
}




pub fn print_patching_variable(name: &str, value: &Atom) {
    showln!(cyan_bold, "• ", gray_dim, name, cyan_bold, " » ", white, value);
}

pub fn show_error(line: &str) {
    let mut remaining_line = line.to_string();
    while remaining_line.len() > 56 {
        let (first, second) = remaining_line.split_at(56);
        showln!(red_bold, "│ ", gray_dim, first);
        remaining_line = second.to_string();
    }
    showln!(red_bold, "│ ", gray, remaining_line);
}


pub fn truncate_line(line: &str, max_chars: usize) -> String {
    if line.trim().len() > max_chars {
        format!("{}...", &line.trim()[..max_chars])
    } else {
        line.trim().to_string()
    }
}

pub fn truncate_interpolatable_line(line: InterpolatedString, max_chars: usize) -> String {
    let line = line.to_string();
    if line.trim().len() > max_chars {
        format!("{}...", &line.trim()[..max_chars])
    } else {
        line.trim().to_string()
    }
}



pub fn format_elapsed_time(elapsed: std::time::Duration) -> String {
    if elapsed.as_secs() > 0 {
        format!("{}s", elapsed.as_secs())
    } else {
        format!("{}ms", elapsed.as_millis())
    }
}

pub fn print_elapsed_time(elapsed: String) {
    let len = 60 - elapsed.len() - 3;
    showln!(
        white,
        "╰─",
        white,
        "─".repeat(len),
        gray,
        " ",
        yellow_bold,
        elapsed
    );
}
