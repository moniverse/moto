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
    tasks
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

// pub async fn get_options() -> Vec<AsyncChoice> {
//     let mut choices = vec![];
//     for package in get_packages().await {
//         for task in package.tasks() {
//             let task = task.clone();
//             choices.push(AsyncChoice::new(
//                 task.name(),
//                 task.runtime(),
//                 Arc::new(move || {
//                     let task = task.clone();
//                     Pin::from(Box::new(async move {
//                         runtime::execute(task.get_code(), task.runtime(), "run".to_string())
//                             .await
//                             .unwrap();
//                     }))
//                 }),
//                 package.name().to_string(),
//             ));
//         }
//     }

//     choices.push(AsyncChoice::new(
//         "exit",
//         "exit the program",
//         Arc::new(move || {
//             Pin::from(Box::new(async move {
//                 std::process::exit(0);
//             }))
//         }),
//         "".to_string(),
//     ));

//     choices
// }





pub async fn set(cell: impl Into<Cell>) {
    CTX.children.clone().lock().await.push(cell.into());
}

#[derive(Clone, Debug)]
pub struct Ctx {
    pub variables: Arc<Mutex<Vec<Variable>>>,
    pub children: Arc<Mutex<Vec<Cell>>>,
}

impl Ctx {
    pub fn empty() -> Self {
        Ctx {
            variables: Arc::new(Mutex::new(vec![])),
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
