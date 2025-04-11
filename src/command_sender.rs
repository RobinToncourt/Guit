#![allow(dead_code)]
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuitSendError {
    #[error("Can't send command.")]
    CommandNotSent(std::io::Error),
    #[error("Can't convert to UTF8.")]
    ConvertToUtf8(std::string::FromUtf8Error),
}

pub fn send_log() -> Result<String, GuitSendError> {
    let output = Command::new("git")
        .arg("log")
        .output()
        .map_err(GuitSendError::CommandNotSent)?;

    let result = output.stdout;
    let s = String::from_utf8(result)
        .map_err(GuitSendError::ConvertToUtf8)?;

    Ok(s)
}
