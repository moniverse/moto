use crate::runtime;

use super::*;
use futures::{Future, FutureExt};
use minimo::choice;
use std::collections::HashMap;
use std::{env, fs};
use std::fmt::Result;
use std::io::Write;
use std::path::Path;
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::Arc;

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

pub async fn get_packages() -> Vec<Package> {
    CTX.children
        .clone()
        .lock()
        .await
        .iter()
        .filter_map(|cell| match cell {
            Cell::Package(package) => Some(package.clone()),
            _ => None,
        })
        .collect()
}

pub async fn get_tasks() -> Vec<Task> {
   let mut tasks = vec![];
   for tsk in CTX.children.clone().lock().await.iter() {
       match tsk {
           Cell::Task(task) => tasks.push(task.clone()),
           _ => {}
       }
    }
    for package in get_packages().await {
        tasks.extend(package.tasks());
    }

    //check if moto is installed. if not diplay an option to install it
    if is_moto_installed().await == false {
        tasks.push(Task::new("install moto", "[:install_moto()]","moto"));
        
    }
    tasks
}

pub async fn is_moto_installed() -> bool {
    let user_dir = env::var("USERPROFILE").unwrap();
    let moto_dir = format!("{}\\moto", user_dir);
    let moto_exe = format!("{}\\moto.exe", moto_dir);
    Path::new(&moto_exe).exists()
}

pub async fn install_moto() {
    // Copy the moto binary to user/.moto directory
    let user_dir = env::var("USERPROFILE").unwrap();
    let moto_dir = format!("{}\\moto", user_dir);
    let moto_exe = format!("{}\\moto.exe",  moto_dir);

    // Create the moto directory
    if !Path::new(&moto_dir).exists() {
        fs::create_dir_all(&moto_dir).unwrap();
    }

    // Copy the moto binary to the moto directory
    let self_exe = env::current_exe().unwrap().to_path_buf();
    if !Path::new(&moto_exe).exists() {
        fs::copy(self_exe, &moto_exe).unwrap();
    }

    // Add the path to PATH
    let path_var = "PATH";
    let mut path_value = String::from(env::var(path_var).unwrap());
    path_value.push_str(";");
    path_value.push_str(&moto_dir);
    env::set_var(path_var, &path_value);
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
            Cell::Package(package) => package
                .runtimes()
                .iter()
                .filter_map(|runtime| {
                    if runtime.identifer.matches(&name) {
                        Some(runtime.clone())
                    } else {
                        None
                    }
                })
                .next(),
            _ => None,
        })
        .next()
}

pub async fn get_variable(name: impl Into<String>) -> Option<Atom> {
    let name = name.into().trim().to_lowercase();
    CTX.variables.clone().lock().await.get(&name).cloned()
}

pub async fn get_variable_or_default(name: impl Into<String>, default: impl Into<Atom>) -> Atom {
    get_variable(name).await.unwrap_or(default.into())
}

pub async fn get_function(name: impl Into<String>) -> Option<Task> {
    let name = name.into();
    CTX.children
        .clone()
        .lock()
        .await
        .iter()
        .filter_map(|cell| match cell {
            Cell::Task(function) => {
                if function.identifer.matches(&name) {
                    Some(function.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
}

pub type Fx = Arc<fn() ->  Pin<Box<dyn Future<Output = ()> + Send>> >;

lazy_static::lazy_static!{
    pub static ref INTERNAL_FUNCTIONS: HashMap<String,  Fx> = {
        let mut map = HashMap::<String, Fx>::new();
        map.insert("install moto".to_string(), Arc::new(|| Box::pin(install_moto())) );
        map
    };
}

pub async fn get_internal_function(name: impl Into<String>) -> Option<Fx> {
    let name = name.into();
    INTERNAL_FUNCTIONS.get(&name).cloned()
}



pub async fn set_variable(name: impl Into<String>, value: Atom) {
    let name = name.into().trim().to_lowercase();
    print_setting_variable(&name, &value);
    CTX.variables.clone().lock().await.insert(name, value);
}

fn print_setting_variable(name: &str, value: &Atom) {
    showln!(cyan_bold, "• ", gray_dim, name, cyan_bold, " » ", white, value);
}




pub async fn push_cell(cell: impl Into<Cell>) {
    CTX.children.clone().lock().await.push(cell.into());
}

#[derive(Clone, Debug)]
pub struct Ctx {
    pub variables: Arc<Mutex<HashMap<String, Atom>>>,
    pub children: Arc<Mutex<Vec<Cell>>>,
}

impl Ctx {
    pub fn empty() -> Self {
        Ctx {
            variables: Arc::new(Mutex::new(HashMap::new())),
            children: Arc::new(Mutex::new(vec![])),
        }
    }
}

pub async fn get_configurations() -> Vec<AsyncChoice> {
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
            "".to_string(),
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
            "".to_string(),
        ),
    ]
}

pub fn get_local_repository_path(name: &str) -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let path = current_dir.join("repositories").join(name);
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}
