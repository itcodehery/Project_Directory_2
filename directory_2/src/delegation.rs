use std::process::{Command, Stdio};

pub fn execute_with_piping(input: &str) -> Result<(), String> {
    #[cfg(windows)]
    let status_res = Command::new("cmd")
        .args(&["/C", input])
        .current_dir("./")
        .status();

    #[cfg(unix)]
    let status_res = Command::new("sh")
        .args(&["-c", input])
        .current_dir("./")
        .status();

    match status_res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
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

    #[cfg(unix)]
    let status = Command::new("sh")
        .args(&["-c", input])
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
                    status.code().unwrap_or(-1)
                ));
            }
        }
        Err(e) => Err(format!("Command execution failed: {}", e)),
    }
}
