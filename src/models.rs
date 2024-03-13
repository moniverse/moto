// use std::{collections::HashMap, fmt::Formatter, str::from_utf8};
// use tokio::process::Command;

// use tokio::io::AsyncBufReadExt;
// use kdam::{Bar, Animation, BarExt};

// use crate::progress::{PROGRESS, ExBar};


// pub struct Printer {
//     pub stdio: Bar,
// }

// impl Printer {
//     pub fn new() -> Self {
//         Self {
//             stdio: kdam::tqdm!(
//                 ncols = 40_i16,
//                 position = 2,
//                 force_refresh = true,
//                 animation = Animation::custom_with_fill(&["─", "─"], "⋅"),
//                 spinner = kdam::Spinner::new(&[
//                     "🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘", "🌑", "⚽", "🌏", "🌍", "🌎", "🌏", "🌐", "🍄", "🌺",
//                     "🌈", "🌙", "🔥", "💧", "🌳", "🌷", "🌸", "🌹", "🌻", "🌼", "🍉", "🍊", "🍋", "🍌", "🍍", "🍎", "🍏",
//                     "🍐", "🍑", "🍒", "🍓", "🍔", "🍕", "🍖",
//                 ], 80.0, 0.5),
//                 colour = "gradient(#ECECEC,#EE6FF8,#5A56E0,#00FFFF)",
//                 bar_format = " ╰─ {spinner}{desc} {animation} {elapsed human=true} {count}/{total}"
//             ),
//         }
//     }

//     pub fn print(&mut self, message: &str) {
//         self.stdio.write(message);
//     }
// }