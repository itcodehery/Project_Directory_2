#[cfg(windows)]
use std::process::{Command, Stdio};

pub fn execute_with_piping(input: &str) -> Result<(), String> {
    #[cfg(windows)]
    let _ = Command::new("cmd")
        .args(&["/C", input])
        .current_dir("./")
        .status();

    #[cfg(unix)]
    let _ = Command::new("sh")
        .args(&["-c", input])
        .current_dir("./")
        .status()?;
    Ok(())
}

pub fn execute_using_cmd(input: &str) -> Result<(), String> {
    #[cfg(windows)]
    let status = Command::new("cmd")
        .args(&["/C", input])
        .current_dir("./")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                return Ok(());
            } else {
                return Err(format!(
                    "Command failed with exit code: {}",
                    status.code().unwrap()
                ));
            }
        }
        Err(e) => Err(format!("Command execution failed: {}", e)),
    }
    // Ok(())
}
