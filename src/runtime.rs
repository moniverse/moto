use std::pin::Pin;

use crate::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::pin;
use tokio::process::Command;

use crate::get_runtime;

pub async fn execute(
    code: impl Into<String>,
    runtime: impl Into<String>,
    runtime_task: impl Into<String>,
) -> Result<String, String> {
    let code = code.into();
    let runtime = runtime.into();
    let runtime_task = runtime_task.into();
    let workspace = get_workspace().await;
    let instance = std::time::Instant::now();

    match runtime.as_str() {
        "shell" | "sh" | "powershell" | "ps" => {
            execute_simple_runtime(code, &runtime, &workspace).await
        }
        _ => execute_complex_runtime(code, &runtime, &runtime_task, &workspace).await,
    }?;

    let elapsed = format_elapsed_time(instance.elapsed());
    print_elapsed_time(elapsed);

    Ok("".into())
}

async fn execute_simple_runtime(
    code: impl Into<String>,
    runtime: &str,
    workspace: &str,
) -> Result<(), String> {
    let (command, arg) = match runtime {
        "shell" | "sh" => ("bash", "-c"),
        "powershell" | "ps" => ("pwsh", "-Command"),
        _ => return Err(format!("Unsupported simple runtime: {}", runtime)),
    };

    let mut child = spawn_child_process(command, arg, workspace)?;
    let mut stdin = child.stdin.take().expect("failed to get stdin");
    let stdout = child.stdout.take().expect("failed to get stdout");
    let stderr = child.stderr.take().expect("failed to get stderr");
    let code = code.into();

    for line in code.lines() {
        let displayable = truncate_interpolatable_line(line.to_string(), 50);
        if !displayable.is_empty() {
            // showln!(yellow_bold, "↓ ", gray_dim, displayable);
            showln!(yellow_bold, "⇣ ", gray_dim, displayable);
        }

        let line = dope(line.clone()).await;

        write_to_stdin(&mut stdin, &line).await?;
    }

    std::mem::drop(stdin);

    process_output(stdout, stderr).await;

    child.wait().await.expect("failed to wait on child");

    Ok(())
}

async fn execute_complex_runtime(
    code: impl Into<String>,
    runtime: &str,
    runtime_task: &str,
    workspace: &str,
) -> Result<(), String> {
    let runtime = get_runtime(runtime.to_string())
        .await
        .ok_or_else(|| format!("runtime {} not found", runtime))?;
    let task = runtime.get_task(runtime_task.to_string()).ok_or_else(|| {
        format!(
            "task {} not found in runtime {}",
            runtime_task,
            runtime.name()
        )
    })?;

    let runtime = task.runtime();

    let (command, arg) = match runtime.as_str() {
        "shell" | "sh" => ("bash", "-c"),
        "powershell" | "ps" => ("pwsh", "-Command"),
        _ => return Err(format!("Unsupported runtime: {}", runtime)),
    };

    let mut child = spawn_child_process(command, arg, workspace)?;
    let mut stdin = child.stdin.take().expect("failed to get stdin");
    let stdout = child.stdout.take().expect("failed to get stdout");
    let stderr = child.stderr.take().expect("failed to get stderr");

    execute_task_code(&task, code, &mut stdin).await?;

    stdin.flush().await.expect("failed to flush stdin");
    std::mem::drop(stdin);
    process_output(stdout, stderr).await;

    child.wait().await.expect("failed to wait on child");

    Ok(())
}

fn spawn_child_process(
    command: &str,
    arg: &str,
    workspace: &str,
) -> Result<tokio::process::Child, String> {
    Command::new(command)
        .arg(arg)
        .current_dir(workspace)
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| "failed to execute child".to_string())
}

async fn execute_task_code(
    task: &Task,
    code: impl Into<String>,
    stdin: &mut tokio::process::ChildStdin,
) -> Result<(), String> {
    let code = code.into();
    let block_code = dope(code.clone()).await;
    let block_code = block_code.trim();
    set_variable("block", block_code.into()).await;
    for line in task.get_code().lines().skip_while(|line| line.is_empty()) {
        let displayable = truncate_interpolatable_line(line.to_string(), 50);
        if !displayable.is_empty() {
            // showln!(yellow_bold, "↓ ", gray_dim, displayable);
            showln!(yellow_bold, "⇣ ", gray_dim, displayable);
        }

        let line = dope(line).await;



        write_to_stdin(stdin, &line).await?;
    }

    // Take ownership of stdin and drop it
    let _ = stdin.flush().await;

    Ok(())
}

async fn write_to_stdin(stdin: &mut tokio::process::ChildStdin, line: &str) -> Result<(), String> {
    stdin
        .write_all(line.as_bytes())
        .await
        .map_err(|_| "failed to write to stdin".to_string())?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(|_| "failed to write newline to stdin".to_string())
}

async fn process_output(stdout: tokio::process::ChildStdout, stderr: tokio::process::ChildStderr) {
    let mut stdout_lines = BufReader::new(stdout).lines();
    while let Some(line) = stdout_lines.next_line().await.unwrap() {
        show_output(&line);
    }

    let mut stderr_lines = BufReader::new(stderr).lines();
    while let Some(line) = stderr_lines.next_line().await.unwrap() {
        show_error(&line);
    }
}

fn show_output(line: &str) {
    let mut remaining_line = line.to_string();
    while remaining_line.len() > 56 {
        let (first, second) = remaining_line.split_at(56);
        showln!(white, "│ ", white, first);
        remaining_line = second.to_string();
    }
    showln!(white, "│ ", white, remaining_line);
}

pub async fn get_workspace() -> String {
    let current_dir = std::env::current_dir().unwrap();
    let workspace = current_dir.join("workspace");
    if !workspace.exists() {
        tokio::fs::create_dir_all(&workspace)
            .await
            .expect("failed to create workspace");
    }
    workspace.to_str().unwrap().to_string()
}

impl Runtime {
    pub fn get_task(&self, name: impl Into<String>) -> Option<Task> {
        let name = name.into();
        self.children
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
}


/// dope
/// doping is the process of identifying and replacing interpolatable variables in a string
/// with their current values (from ctx)
/// variables are identified by the following pattern: [:variable_name] or [:variable_name=default_value] or [:variable_name(arg1, arg2, ...)]
/// for example: "Hello, [:name]!" will be replaced with "Hello, John!" if ctx contains a variable named "name" with value "John"
/// and "Hello, [:name=John]!" will be replaced with "Hello, John!" if ctx does not contain a variable named "name"
/// and "Hello, [:name(John, Doe)]!" will be replaced with "Hello, how are you john the son of Doe?" if ctx has a function named "name" that takes two arguments
/// and returns "how are you [0] the son of [1]?"
/// and "Hello, [:name(John)]!" will be replaced with "Hello, John!" if ctx has a function named "name" that takes one argument
/// and returns "Hello, [0]!"
pub async fn dope(code: impl Into<String>) -> String {

    let mut code = code.into();

    //sequentially identify and replace interpolatable variables and functions in the code
    while let Some((start, end)) = find_interpolatable(&code) {
        match code[start..end].find('(') {
            Some(open) => {
                let close = code[start..end].find(')').unwrap();
                let (name, args) = code[start..end].split_at(open);
                let args = &args[1..close];
                let args = args.split(',').map(|arg| arg.trim()).collect::<Vec<&str>>();
                let value = get_function_value(name, args).await;
                code.replace_range(start..end, &value);
            }
            None => {
                match code[start..end].find('=') {
                    Some(equal) => {
                        let (name, default) = code[start..end].split_at(equal);
                        let default = &default[1..].trim_end_matches(']');
                        let value = get_variable_value(name, default).await;
                        code.replace_range(start..end, &value.to_string());
                    }
                    None => {
                        let value = get_variable_value(&code[start..end], "").await;
                        code.replace_range(start..end, &value.to_string());
                    }
                }
            }
        }

    }

    code

}


/// find_interpolatable
/// find the next interpolatable variable or function in the code
/// identified by the following pattern: [:variable_name] or [:variable_name=default_value] or [:variable_name(arg1, arg2, ...)]
/// for example: "Hello, [:name]!" has [:name] as an interpolatable variable starting at index 7 and ending at index 13
pub fn find_interpolatable(code: &str) -> Option<(usize, usize)> {
    let start = code.find("[:");
    let start = match start {
        Some(start) => start,
        None => return None,
    };

    let end = code[start..].find(']');
    let end = match end {
        Some(end) => start + end + 1,
        None => return None,
    };

    Some((start, end))
}

/// get_variable_value
/// get the value of a variable from the context
/// if the variable is not found in the context, return the default value
pub async fn get_variable_value(name: &str, default: &str) -> Atom {
    let name = name.trim_start_matches("[:").trim_end_matches("]");
    let default = default.trim();
    let value = get_variable(name).await;
    match value {
        Some(value) => value,
        None => default.into(),
    }
}

/// get_function_value
/// get the value of a function from the context
/// if the function is not found in the context, return an empty string
/// if the function is found in the context, call the function with the arguments and return the result
pub async fn get_function_value(name: &str, args: Vec<&str>) -> String {
    let name = name.trim_start_matches("[:").trim_end_matches("]");
    let value = get_function(name).await;
    match value {
        Some(value) => {
            let runtime = value.runtime();
            let runtime_task = value.name();
            let code = value.get_code();
          " function not implemented".into()
        }
        None => "".into(),
    }
}
