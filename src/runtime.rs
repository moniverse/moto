use crate::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use crate::get_runtime;

pub async fn execute(
    code: impl Into<InterpolatedString>,
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
    code: impl Into<InterpolatedString>,
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
        let displayable = truncate_interpolatable_line(line.clone(), 50);
        if !displayable.is_empty() {
            // showln!(yellow_bold, "↓ ", gray_dim, displayable);
            showln!(yellow_bold, "⇣ ", gray_dim, displayable);
        }

        let line = inject_variables(line.clone()).await;

        write_to_stdin(&mut stdin, &line).await?;
    }

    std::mem::drop(stdin);

    process_output(stdout, stderr).await;

    child.wait().await.expect("failed to wait on child");

    Ok(())
}

async fn execute_complex_runtime(
    code: impl Into<InterpolatedString>,
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
    code: impl Into<InterpolatedString>,
    stdin: &mut tokio::process::ChildStdin,
) -> Result<(), String> {
    let code = code.into();
    for line in task.get_code().lines().skip_while(|line| line.is_empty()) {
        let displayable = truncate_interpolatable_line(line.clone(), 50);
        if !displayable.is_empty() {
            // showln!(yellow_bold, "↓ ", gray_dim, displayable);
            showln!(yellow_bold, "⇣ ", gray_dim, displayable);
        }

        let line = inject_variables(line.clone()).await;
        let injcode = inject_variables(code.clone()).await;
        let line = inject_block_code(&line, &injcode);

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


pub async fn inject_variables(code: impl Into<InterpolatedString>) -> String {
    let mut code = code.into();
    while code.is_computable() {
        code = code.decompose().compute().await;
    }
    code.to_string()
}

fn inject_block_code(line: &str, code: &str) -> String {
    line.replace("[:code]", code).replace("[:block]", code)
}
