use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::*;

use crate::{get_runtime, get_variables}; 




pub async fn execute(
    code: impl Into<String>,
    runtime: impl Into<String>,
    runtime_task: impl Into<String>,
) -> Result<String, String> {
    let code = code.into();
    let runtime = runtime.into();
    let runtime_task = runtime_task.into();
    let workspace = get_workspace().await;
    let runtime = get_runtime(runtime.clone()).await.ok_or(format!("runtime {} not found", runtime))?;
    let task = runtime.get_task(runtime_task.clone()).ok_or(format!(
        "task {} not found in runtime {}",
        runtime_task, runtime.name()
    ))?;

    // Inject variables into task code and actual code
    let variables = get_variables().await;

    let mut child = Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-Command")
        .current_dir(&workspace)
        .arg("-") // Read command from stdin
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    let mut stdin = child.stdin.take().expect("failed to get stdin");
    let stdout = child.stdout.take().expect("failed to get stdout"); 
    let stderr = child.stderr.take().expect("failed to get stderr");

    let instance = std::time::Instant::now();
    // Execute the task code line by line
    for line in task.get_code().lines().skip_while(|line| line.trim().is_empty()) {
        let code = code.clone();
        let displayable = //first 20 chars of the line
            if line.trim().len() > 20 {
                format!("{}...", &line.trim()[..20])
            } else {
                line.trim().to_string()
            };
        showln!(yellow_bold,"╰",white_bold,"→ ",gray_dim, displayable);

        //inject variables into the line
        let line = inject_variables(line, &variables);
        let line = inject_variables(line, &vec![Variable::new("block", code)]);
        
        // Write the command to stdin
        stdin.write_all(line.as_bytes()).await.expect("failed to write to stdin");
        stdin.write_all(b"\n").await.expect("failed to write newline to stdin");
        
      
       
    }
    
    // Close stdin to signal end of input
    std::mem::drop(stdin);

    // Process stdout
    let mut stdout_lines = BufReader::new(stdout).lines();
    while let Some(line) = stdout_lines.next_line().await.unwrap() {
        //wrap output to max 60 chars per line
        let mut line = line;
        while line.len() > 56 {
            let (first, second) = line.split_at(60);
            showln!(white,"│ ",gray_dim,first);
            line = second.to_string();
        }
        showln!(white, "│ ", gray_dim, line);
    }

    // Process stderr
    let mut stderr_lines = BufReader::new(stderr).lines();
    while let Some(line) = stderr_lines.next_line().await.unwrap() {
        //wrap output to max 60 chars per line 
        let mut line = line;
        while line.len() > 58 {
            let (first, second) = line.split_at(60);
            showln!(red_bold,"│ ",red_dim,first);    
            line = second.to_string();
        }
        showln!(red_bold, "│ ", red_dim, line);
    }

    // Wait for the child process to finish
    let _ = child.wait().await.expect("failed to wait on child");

    let elapsed = instance.elapsed();
    let elapsed = // like 2.3s
        if elapsed.as_secs() > 0 {
            format!("{}s", elapsed.as_secs()) 
        } else {
            format!("{}ms", elapsed.as_millis())
        };
    let len = 60-elapsed.len() -3;
    showln!(white,"╰─", white, "─".repeat(len), gray, " ", yellow_bold, elapsed);

    Ok("".into())
}

pub async fn get_workspace() -> String {
    let current_dir = std::env::current_dir().unwrap();
    let workspace = current_dir.join("workspace");
    if !workspace.exists() {
        tokio::fs::create_dir_all(&workspace).await.expect("failed to create workspace");
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

pub fn inject_variables(code: impl Into<String>, variables: &[Variable]) -> String {
    let mut code = code.into();
    let mut modified_code = String::new();
    for variable in variables {
        let placeholder = format!("[:{}]", variable.name());
        let mut start_pos = 0;
        while let Some(start) = code[start_pos..].find(&placeholder) {
            let start = start + start_pos;
            let end = code[start..]
                .find(']')
                .map(|i| i + start)
                .unwrap_or_else(|| code.len());
            let placeholder = &code[start..=end];
            let default_value = placeholder.find('=').map(|start| &placeholder[start+1..]).unwrap_or("");
            let value = variable.get_value_or(default_value.into());
            modified_code.push_str(&code[start_pos..start]);
            modified_code.push_str(&value.to_string());
            start_pos = end + 1;
        }
        modified_code.push_str(&code[start_pos..]);
        code = modified_code.clone();
        modified_code.clear();
    }
    code
}
