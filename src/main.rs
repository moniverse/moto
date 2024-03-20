#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(unused_assignments)]
#[allow(dead_code)]
#[allow(unused_must_use)]
#[allow(unused_parens)]
// Parse the input file
mod parser;


use crossterm::terminal::{ClearType, LeaveAlternateScreen};
use minimo::*;
use moto::*;
use std::{env, fs, path::Path};

#[tokio::main]
async fn main() {
    print_banner();
    start().await;
}

pub async fn start() {
    moto::menu::scan().await.unwrap();
    match moto::menu::handle_args().await {
        Some(action) => {
            action.run().await;
        }
        None => {
            moto::menu::display_options().await.run().await;
        }
    }
}
