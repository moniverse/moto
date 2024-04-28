use std::pin::Pin;

use crate::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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
    let instance = std::time::Instant::now();

    if runtime == "moto" {
        execute_internal(&code, &runtime, &runtime_task).await?;
    } else {
        match runtime.as_str() {
            "shell" | "sh" | "powershell" | "ps" => execute_simple_runtime(&code, &runtime).await,
            _ => execute_complex_runtime(&code, &runtime, &runtime_task).await,
        }?;
    }

    let elapsed = format_elapsed_time(instance.elapsed());
    print_elapsed_time(elapsed);

    Ok("".into())
}

async fn execute_internal(code: &str, runtime: &str, runtime_task: &str) -> Result<(), String> {
    if let Some(fx) = get_internal_function(runtime_task).await {
        showln!(green_bold, "⇣ ", gray_dim, "executing internal function");
        let _ = fx().await;
    } else {
        showln!(red_bold, "⇣ ", gray_dim, "Function not found");
    }
    Ok(())
}

async fn execute_simple_runtime(code: &str, runtime: &str) -> Result<(), String> {
    let (command, arg) = match runtime {
        "shell" | "sh" => ("bash", "-c"),
        "powershell" | "ps" => ("pwsh", "-Command"),
        _ => return Err(format!("Unsupported simple runtime: {}", runtime)),
    };

    execute_runtime(command, arg, code).await
}

async fn execute_complex_runtime(
    code: &str,
    runtime: &str,
    runtime_task: &str,
) -> Result<(), String> {
    let runtime = get_runtime(runtime.to_string())
        .await
        .ok_or_else(|| format!("runtime {} not found", runtime))?;
    let task = runtime
        .get_task(runtime_task)
        .ok_or_else(|| format!("task {} not found in runtime {}", runtime_task, runtime.name()))?;

    let runtime = task.runtime();
    let (command, arg) = match runtime.as_str() {
        "shell" | "sh" => ("bash", "-c"),
        "powershell" | "ps" => ("pwsh", "-Command"),
        _ => return Err(format!("Unsupported runtime: {}", runtime)),
    };

    let mut child = spawn_child_process(command, arg)?;
    let mut stdin = child.stdin.take().expect("failed to get stdin");

    execute_task_code(&task, code, &mut stdin).await?;

    stdin.flush().await.expect("failed to flush stdin");
    drop(stdin);

    child.wait().await.expect("failed to wait on child");

    Ok(())
}

async fn execute_runtime(command: &str, arg: &str, code: &str) -> Result<(), String> {
    let mut child = spawn_child_process(command, arg)?;
    let mut stdin = child.stdin.take().expect("failed to get stdin");
    let stdout = child.stdout.take().expect("failed to get stdout");
    let stderr = child.stderr.take().expect("failed to get stderr");
    let lines = code.lines().map(|line| line.to_string()).collect::<Vec<String>>();
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let output_processor = tokio::spawn(async move {
        while let Some(line) = stdout_reader.next_line().await.unwrap() {
            show_output(&line);
        }
    });

    let error_processor = tokio::spawn(async move {
        while let Some(line) = stderr_reader.next_line().await.unwrap() {
            show_error(&line);
        }
    });

    let input_processor = tokio::spawn(async move {
        for line in lines {
            let displayable = truncate_interpolatable_line(line.to_string(), 50);
            if !displayable.is_empty() {
                showln!(yellow_bold, "⇣ ", gray_dim, displayable);
            }

            let line = dope(line.to_string()).await;

            write_to_stdin(&mut stdin, &line).await.expect("failed to write to stdin");
        }

    });

    tokio::try_join!(output_processor, error_processor, input_processor).map_err(|_| "failed to execute runtime".to_string())?;

    child.wait().await.expect("failed to wait on child");

    Ok(())
}

fn spawn_child_process(command: &str, arg: &str) -> Result<tokio::process::Child, String> {
    Command::new(command)
        .arg("-NoProfile")
        .arg(arg)
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
                Cell::Task(task) if task.identifer.matches(name) => Some(task.clone()),
                _ => None,
            })
            .next()
    }
}

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
                    let name = name.trim_start_matches("[:").trim_end_matches("]");
                    get_variable_or_default(name, *default).await.to_string()
                }
                None => {
                    let name = segment.trim_start_matches("[:").trim_end_matches("]");
                    get_variable_or_default(name, "").await.to_string()
                }
            },
        };

        result.push_str(&value);
        start += end;
    }

    result.push_str(&code[start..]);
    result
}

fn find_interpolatable(code: &str) -> Option<(usize, usize)> {
    let start = code.find("[:")?;
    let end = code[start..].find(']')?;
    Some((start, start + end + 1))
}

async fn get_function_value(name: &str, args: Vec<&str>) -> String {
    let name = name.trim_start_matches("[:").trim_end_matches("]");
    if let Some(value) = get_function(name).await {
        let runtime = value.runtime();
        let runtime_task = value.name();
        let code = value.get_code();
        // function not implemented
        "".into()
    } else {
        "".into()
    }
}