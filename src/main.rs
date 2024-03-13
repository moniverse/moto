

#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(unused_assignments)]
#[allow(dead_code)]
#[allow(unused_must_use)]
#[allow(unused_parens)]



// Parse the input file
mod parser;

pub mod models;
pub mod progress;
use crossterm::terminal::{ClearType, LeaveAlternateScreen};
use moto::*;
use std::{env, fs, path::Path};
use minimo::*;



#[tokio::main]
async fn  main() {
  start().await;
}








pub async fn start() {
let args: Vec<String> = env::args().collect();
moto::ctx::scan().await.unwrap();
get_ctx().display_options().await;
}

