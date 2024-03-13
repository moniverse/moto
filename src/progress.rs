// use std::process::Command;
// #[allow(unused_imports)]
// #[allow(unused_variables)]
// #[allow(unused_mut)]
// #[allow(unused_assignments)]
// #[allow(dead_code)]
// #[allow(unused_must_use)]
// #[allow(unused_parens)]

// use std::sync::OnceLock;




// use kdam::term::Colorizer;
// use kdam::{Bar, Animation};
// use kdam::{BarExt, Spinner};
// use crate::{show, showln};
// use tokio::io::AsyncBufReadExt;
// use tokio::io::BufReader;


// pub static PROGRESS: OnceLock<std::sync::Mutex<Bar>> = OnceLock::new();

// pub static  CTX : OnceLock<std::sync::Mutex<Ctx>> = OnceLock::new();


// pub async fn execute_task(task: &Task) {

//   let mut pb = PROGRESS.get_or_init(|| {
//     let spin = Spinner::new(
//         &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘", "🌑", "⚽", "🌏", "🌍", "🌎", "🌏", "🌐", "🍄", "🌺", "🌈", "🌙", "🔥", "💧",  "🌳", "🌷", "🌸", "🌹", "🌻", "🌼",  "🍉", "🍊", "🍋", "🍌", "🍍", "🍎", "🍏", "🍐", "🍑"],
//         30.0,
//         1.0,
//     );
//     let pb = kdam::tqdm!(
//         ncols = 40_i16,
//         position = 2,
//         total = task.actions.len(),  
//         force_refresh = true,
//         animation = Animation::custom_with_fill(&["━","━"], "⋅"),
//         spinner = spin,
//         colour = "gradient(#ECECEC,#EE6FF8,#5A56E0,#00FFFF)",
//         bar_format = "╰─ {spinner}{desc} {animation} {elapsed human=true} {count}/{total}"
//     );
//         std::sync::Mutex::new(pb)
//     }).lock().unwrap();

//     println!();
//     //opposite ╰─ ╭─
//     // termo::tree::head(&task.name, format!("{} ",&task.description).as_str());
//     showln!(gray_dim,"╭─ ",cyan_bold, &task.name, cyan, " ", &task.description);
//     showln!(gray_dim, "| ");

//     pb.update(0);
//     pb.print("");
//     pb.set_position(1);

//     drop(pb);


//     let mut is_warning = false;

//     for (index, action) in task.actions.iter().enumerate() {
//         let mut log = PROGRESS.get().unwrap().lock().unwrap();

//         let cmd = tokio::process::Command::new(&action.binary)
//             // Additional args for PowerShell to make it run the code smoothly
//             .arg("-NoProfile")
//             .arg("-ExecutionPolicy")
//             .arg("Bypass")
//             .arg("-Command")
//             .arg(&action.code)
//             .stdout(std::process::Stdio::piped())
//             .stderr(std::process::Stdio::piped())
//             .kill_on_drop(true) // Kill the child process when it's dropped

//             .spawn();

//         match cmd {
//             Ok(mut child) => {
//                 let stdout = child.stdout.take().unwrap();
//                 let stderr = child.stderr.take().unwrap();

//                 let stdout_lines = tokio::io::BufReader::new(stdout).lines();
//                 let stderr_lines = tokio::io::BufReader::new(stderr).lines();

//                 tokio::pin!(stdout_lines);
//                 tokio::pin!(stderr_lines);

//                 while let Some(line) = stdout_lines.next_line().await.unwrap() {
//                     log.print_info(line.as_str());
//                     if is_warning {
//                         is_warning = false;
//                     }
//                 }

//                 while let Some(line) = stderr_lines.next_line().await.unwrap() {
//                     if line.to_lowercase().contains("warning")  && !is_warning {
//                         is_warning = true;
//                     }

//                     if line.to_lowercase().contains("error") {
//                         is_warning = false;
//                     }
                
//                     if is_warning {
//                         log.print_warning(line.as_str());
//                     } else {
//                         log.print_error(line.as_str());
//                     }


//                 }

//                 let status = child.wait().await.unwrap();

//                 if status.success() {
//                     // log.print("action executed successfully");
//                 } else {
//                     log.print_error("action failed");
//                 }

//                 log.update(1);
//             }

//             Err(e) => {
//                 log.print_caution("couldn't run action");
//                 log.print_warning(format!("[{}] [{}]", action.binary, action.code).as_str());
//                 log.print_error(format!("{}", e).as_str());
//             }
//         }

//     }

//     let mut pb = PROGRESS.get().unwrap().lock().unwrap();


//     pb.set_position(0);
//     pb.write(PRE);
//     pb.completed();
//     println!();
//     println!();
//     println!();
    
// }

// pub const PRE: &str = "| ";

// pub const GREY: &str = "#3D3D3D";
// pub const RED: &str = "#FF0000";
// pub const GREEN: &str = "#00FF44";
// pub const CYAN: &str = "#00FFFF";
// pub const BLUE: &str = "#0000FF";
// pub const MAGENTA: &str = "#EE6FF8";
// pub const YELLOW: &str = "#FFFF00";
// pub const WHITE: &str = "#FFFFFF";
// pub const ORANGE: &str = "#FFA500";
// pub const PINK: &str = "#FFC0CB";
// pub const PURPLE: &str = "#5A56E0";

// pub trait ExBar {
//     fn print(&mut self, message: &str);
//     fn para_vivid(&mut self, message: &str, color: &str) {
//         let lines: Vec<String> = message.split('\n')
//             .flat_map(|line| {
//                 if line.len() > 50 {
//                     line.chars()
//                         .collect::<Vec<char>>()
//                         .chunks(50)
//                         .map(|chunk| chunk.iter().collect())
//                         .collect()
//                 } else {
//                     vec![line.to_string()]
//                 }
//             })
//             .collect();

//         let brightest = 150; // Brightest white color
//         let dimmest = 50; // Dimmest white color
//         let step = (brightest - dimmest) / lines.len() as u8;

//         let mut current_color = brightest;
//         for line in lines {
//             self.print_with_color(&line, current_color, color);
//             if current_color > dimmest {
//                 current_color -= step;
//             }
//         }
//     }
    

//     fn print_bright(&mut self, message: &str);
//     fn print_error(&mut self, message: &str);
//     fn print_success(&mut self, message: &str);
//     fn print_warning(&mut self, message: &str);
//     fn print_info(&mut self, message: &str);
//     fn print_with_color(&mut self, message: &str, color: u8, base_color: &str);
//     fn print_caution(&mut self, message: &str);

//     fn print_subtle(&mut self, message: &str);
// }

// impl ExBar for Bar {
//     fn print(&mut self, message: &str) {
//         self.write(format!("{}{}", PRE.colorize(GREY), message.trim().colorize(GREY)));
//     }

//     fn print_bright(&mut self, message: &str) {
//         self.para_vivid(message, WHITE);
//     }

//     fn print_error(&mut self, message: &str) {
//         self.para_vivid(message, RED);
//     }

//     fn print_success(&mut self, message: &str) {
//         self.para_vivid(message, GREEN);
//     }

//     fn print_warning(&mut self, message: &str) {
//         self.para_vivid(message, ORANGE);
//     }

//     fn print_info(&mut self, message: &str) {
//         self.para_vivid(message, CYAN);
//     }

//     fn print_subtle(&mut self, message: &str) {
//         self.para_vivid(message, GREY);
//     }



//     fn print_with_color(&mut self, message: &str, color: u8, base_color: &str) {
//         let hex_color = base_color.to_string();
//         let dim_color = format!("#{:02X}{:02X}{:02X}", color, color, color);
//         self.write(format!("{}{}", PRE.colorize(&hex_color), message.trim().colorize(&hex_color)));
//     }

//     fn print_caution(&mut self, message: &str) {
//         self.para_vivid(message, YELLOW);
//     }
// }


