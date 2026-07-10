use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn execute_with_piping(input: &str) -> Result<(), String> {
    #[cfg(windows)]
    let mut child = tokio::process::Command::new("cmd")
        .args(&["/C", input])
        .current_dir("./")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(unix)]
    let mut child = tokio::process::Command::new("sh")
        .args(&["-c", input])
        .current_dir("./")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = stdout_reader.next_line() => {
                crate::cprintln!("{}", line);
            }
            Ok(Some(line)) = stderr_reader.next_line() => {
                crate::cprintln!("{}", line);
            }
            else => break,
        }
    }
    
    let _ = child.wait().await;
    Ok(())
}
