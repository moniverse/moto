use nom::combinator::complete;
use nom::InputLength;

pub use menu::*;
pub mod menu;
pub use ast::*;
pub use derive_more::*;
pub use minimo::*;
pub mod ast;

pub use parser::*;
pub mod parser;

pub use ctx::*;
pub mod ctx;

pub use runtime::*;
pub mod runtime;

pub use repository::*;
pub mod repository {
    use tokio::fs;

    use super::*;
    ///! repository
    ///! repository is a registrable git repository local / remote (github, gitlab, bitbucket) with some predefined runtimes and tasks
    ///! by adding a repository to the context, the user can access the runtimes and tasks defined in the repository
    ///! moto comes with a default repository that contains some predefined runtimes and tasks [github.com/moniverse/core]
    ///! here common runtimes like rust,dart,javascript,csharp,python,go etc are defined which can be used right out of the box by the user
    ///! the user can also add their own repository to the context

    #[derive(Debug, Display, From, Clone)]
    #[display(fmt = "{}", name)]
    pub struct Repository {
        name: String,
        url: String,
    }

    impl Repository {
        pub fn new(name: String, url: String) -> Self {
            Self { name, url }
        }

        pub fn name(&self) -> String {
            self.name.clone()
        }

        pub fn url(&self) -> String {
            self.url.clone()
        }

        pub async fn load_cells(&self) {
            //if the path is local, look at all the moto files and load all the cells into the context
            //if the path is remote, clone the repository and then load all the cells into the context

            let path = ctx::get_local_repository_path(&self.name);
            
            if !path.exists() {
                self.clone_to(&path).await
            }

             match fs::read_dir(&path).await {
                Ok(mut dir) => {
                    while let Ok(Some(entry)) = dir.next_entry().await {
                        let path = entry.path();
                        if path.extension().unwrap_or_default() == "moto" {
                            let content = fs::read_to_string(&path).await.unwrap();
                            let cells = parser::parse_cells(&content);
                             match cells {
                                Ok((_, cells)) => {
                                    for cell in cells {
                                        ctx::set(cell).await;
                                    }
                                }
                                Err(e) => {
                                    println!("Error: {:?}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }


        pub async fn clone_to(&self, path: &std::path::Path) {
            let _ = fs::create_dir_all(path).await;
            let _ = tokio::process::Command::new("git")
                .args(&["clone", &self.url, path.to_str().unwrap()])
                .output()
                .await;
        }

        pub async fn pull(&self) {
            let path = ctx::get_local_repository_path(&self.name);
            let _ = tokio::process::Command::new("git")
                .current_dir(&path)
                .args(&["pull"])
                .output()
                .await;
        }
    }
}



