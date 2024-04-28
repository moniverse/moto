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

    if runtime == "moto" {
        execute_internal(&code, &runtime, &runtime_task, &workspace).await.expect("failed to execute internal function");
    }

    match runtime.as_str() {
        "shell" | "sh" | "powershell" | "ps" => {
            execute_simple_runtime(&code, &runtime, &workspace).await
        }
        _ => execute_complex_runtime(&code, &runtime, &runtime_task, &workspace).await,
    }?;

    let elapsed = format_elapsed_time(instance.elapsed());
    print_elapsed_time(elapsed);

    Ok("".into())
}

async fn execute_internal(
    code: &str,
    runtime: &str,
    runtime_task: &str,
    workspace: &str,
) -> Result<(), String> {
    //when runtime is moto, the implementation is a rust function that is stored in a hashmap
    let fx = get_internal_function(runtime_task).await;
    match fx {
        Some(fx) => {
            showln!(green_bold, "⇣ ", gray_dim, "executing internal function");
            let result = fx().await;
        }
        None => {
            showln!(red_bold, "⇣ ", gray_dim, "Function not found");
        }
    }
    Ok(())
}

async fn execute_simple_runtime(code: &str, runtime: &str, workspace: &str) -> Result<(), String> {
    let (command, arg) = match runtime {
        "shell" | "sh" => ("bash", "-c"),
        "powershell" | "ps" => ("pwsh", "-Command"),
        _ => return Err(format!("Unsupported simple runtime: {}", runtime)),
    };

    let mut child = spawn_child_process(command, arg, workspace)?;
    let mut stdin = child.stdin.take().expect("failed to get stdin");

    let drain_stdout = child.stdout.take().expect("failed to get stdout");
    let drain_stderr = child.stderr.take().expect("failed to get stderr");

    // Drain the initial stdout output during child process launch
    tokio::spawn(async move {
        drain_initial_output(drain_stdout, drain_stderr).await;
    })
    .await
    .unwrap();

    let stdout = child.stdout.take().expect("failed to get stdout");
    let stderr = child.stderr.take().expect("failed to get stderr");

    let code_lines: Vec<String> = code.lines().map(|line| line.to_string()).collect();

    tokio::spawn(async move {
        process_output_after_first_command(stdout, stderr, code_lines).await;
    });

    for line in code.lines() {
        let displayable = truncate_interpolatable_line(line.to_string(), 50);
        if !displayable.is_empty() {
            showln!(yellow_bold, "⇣ ", gray_dim, displayable);
        }

        let line = dope(line.to_string()).await;

        write_to_stdin(&mut stdin, &line).await?;
    }

    std::mem::drop(stdin);

    child.wait().await.expect("failed to wait on child");

    Ok(())
}

async fn drain_initial_output(
    mut stdout: tokio::process::ChildStdout,
    mut stderr: tokio::process::ChildStderr,
) {
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        // Consume the line from stdout
                        showln!(purple_dim, line);
                    }
                    _ => break,
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        // Consume the line from stderr
                        showln!(red_bold, line);
                    }
                    _ => break,
                }
            }
        }
    }
}

async fn execute_complex_runtime(
    code: &str,
    runtime: &str,
    runtime_task: &str,
    workspace: &str,
) -> Result<(), String> {
    let runtime = get_runtime(runtime.to_string())
        .await
        .ok_or_else(|| format!("runtime {} not found", runtime))?;
    let task = runtime.get_task(runtime_task).ok_or_else(|| {
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

    let drain_stdout = child.stdout.take().expect("failed to get stdout");
    let drain_stderr = child.stderr.take().expect("failed to get stderr");

    // Drain the initial stdout output during child process launch
    tokio::spawn(async move {
        drain_initial_output(drain_stdout, drain_stderr).await;
    })
    .await
    .unwrap();

    let stdout = child.stdout.take().expect("failed to get stdout");
    let stderr = child.stderr.take().expect("failed to get stderr");

    let code_lines: Vec<String> = code.lines().map(|line| line.to_string()).collect();

    tokio::spawn(async move {
        process_output_after_first_command(stdout, stderr, code_lines).await;
    });

    execute_task_code(&task, code, &mut stdin).await?;

    stdin.flush().await.expect("failed to flush stdin");
    std::mem::drop(stdin);

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
    code: &str,
    stdin: &mut tokio::process::ChildStdin,
) -> Result<(), String> {
    let block_code = dope(code.to_string()).await;
    let block_code = block_code.trim();
    set_variable("block", block_code.into()).await;
    for line in task.get_code().lines().skip_while(|line| line.is_empty()) {
        let displayable = truncate_interpolatable_line(line.to_string(), 50);
        if !displayable.is_empty() {
            showln!(yellow_bold, "⇣ ", gray_dim, displayable);
        }

        let line = dope(line.to_string()).await;

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

async fn process_output_after_first_command(
    mut stdout: tokio::process::ChildStdout,
    mut stderr: tokio::process::ChildStderr,
    code_lines: Vec<String>,
) {
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut command_index = 0;

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if command_index < code_lines.len() {
                            if line.trim() == code_lines[command_index].trim() {
                                command_index += 1;
                            } else {
                                show_output(&line);
                            }
                        } else {
                            show_output(&line);
                        }
                    }
                    _ => break,
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if command_index > 0 {
                            show_error(&line);
                        }
                    }
                    _ => break,
                }
            }
        }
    }
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
    pub fn get_task(&self, name: &str) -> Option<Task> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Task(task) => {
                    if task.identifer.matches(name) {
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
pub async fn dope(code: String) -> String {
    let mut result = String::new();
    let mut start = 0;

    while let Some((new_start, end)) = find_interpolatable(&code[start..]) {
        result.push_str(&code[start..start + new_start]);

        let segment = &code[start + new_start..start + end];
        let value = match segment.find('(') {
            Some(open) => {
                let close = segment[open..].find(')').unwrap() + open;
                let (name, args) = segment.split_at(open);
                let args = &args[1..close];
                let args = args.split(',').map(|arg| arg.trim()).collect::<Vec<&str>>();
                get_function_value(name, args).await
            }
            None => match segment.find('=') {
                Some(equal) => {
                    let (name, default) = segment.split_at(equal);
                    let default = &default[1..].trim_end_matches(']');
                    get_variable_value(name, default).await.to_string()
                }
                None => get_variable_value(segment, "").await.to_string(),
            },
        };

        result.push_str(&value);
        start += end;
    }

    result.push_str(&code[start..]);
    result
}

/// find_interpolatable
/// find the next interpolatable variable or function in the code
/// identified by the following pattern: [:variable_name] or [:variable_name=default_value] or [:variable_name(arg1, arg2, ...)]
/// for example: "Hello, [:name]!" has [:name] as an interpolatable variable starting at index 7 and ending at index 13
fn find_interpolatable(code: &str) -> Option<(usize, usize)> {
    let start = code.find("[:")?;
    let end = code[start..].find(']')?;
    Some((start, start + end + 1))
}

/// get_variable_value
/// get the value of a variable from the context
/// if the variable is not found in the context, return the default value
async fn get_variable_value(name: &str, default: &str) -> Atom {
    let name = name.trim_start_matches("[:").trim_end_matches("]");
    let value = runtime::get_variable(name).await;
    value.unwrap_or_else(|| default.into())
}

/// get_function_value
/// get the value of a function from the context
/// if the function is not found in the context, return an empty string
/// if the function is found in the context, call the function with the arguments and return the result
async fn get_function_value(name: &str, args: Vec<&str>) -> String {
    let name = name.trim_start_matches("[:").trim_end_matches("]");
    let value = get_function(name).await;
    match value {
        Some(value) => {
            let runtime = value.runtime();
            let runtime_task = value.name();
            let code = value.get_code();
            // function not implemented
            "".into()
        }
        None => "".into(),
    }
}
