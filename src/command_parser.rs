#![allow(dead_code)]

use std::convert::TryFrom;
use std::sync::LazyLock;
use std::fmt;

use regex::Regex;
use thiserror::Error;

const STATUS_REGEX_STR: &str = {
    r"(?<x>[ MADR])(?<y>[ MADR]) ((?<orig_path>.*) -> )?(?<path>.*)"
};

static STATUS_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(STATUS_REGEX_STR).expect("Invalid regex.")
);

#[derive(Debug, Error)]
enum GuitParseError {
    #[error("Unknown status.")]
    UnknownStatusTable,
    #[error("Malformed status output.")]
    MalformedStatusOutput,
}

struct StatusTableError;

#[derive(Debug, PartialEq)]
enum StatusTable {
    None,
    Unknown,
    Modified,
    Added,
    Deleted,
    Renamed,
}

impl fmt::Display for StatusTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Self::None => " ",
            Self::Unknown => "?",
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
        };

        write!(f, "{c}")
    }
}

impl TryFrom<&str> for StatusTable {
    type Error = GuitParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        assert!(value.len() == 1);

        match value {
            " " => Ok(StatusTable::None),
            "M" => Ok(StatusTable::Modified),
            "A" => Ok(StatusTable::Added),
            "D" => Ok(StatusTable::Deleted),
            "R" => Ok(StatusTable::Renamed),
            _ => Err(GuitParseError::UnknownStatusTable),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Path(String);

#[derive(Debug, PartialEq)]
struct StatusPath {
    x: StatusTable,
    y: StatusTable,
    orig_path: Option<Path>,
    path: Path,
}

#[derive(Debug, PartialEq)]
struct StatusOutput(Vec<StatusPath>);

fn parse_status_output(
    status_output: &str,
) -> Result<StatusOutput, GuitParseError> {
    assert!(status_output.len() >= 4);

    let lines = status_output.split('\n');

    let mut status_output = StatusOutput(
        Vec::with_capacity(lines.size_hint().0)
    );

    for line in lines {
        let Some(caps) = STATUS_REGEX.captures(line) else {
            return Err(GuitParseError::MalformedStatusOutput);
        };

        let x: &str = caps.name("x").ok_or(GuitParseError::MalformedStatusOutput)?.into();
        let x = StatusTable::try_from(x)?;

        let y: &str = caps.name("y").ok_or(GuitParseError::MalformedStatusOutput)?.into();
        let y = StatusTable::try_from(y)?;

        let orig_path: Option<Path> = caps.name("orig_path")
            .map(|s|
                Path(s.as_str().to_owned())
            );

        let path = Path(
            caps.name("path")
            .ok_or(GuitParseError::MalformedStatusOutput)?
            .as_str()
            .to_owned()
        );

        let status_path = StatusPath {
            x,
            y,
            orig_path,
            path,
        };
        status_output.0.push(status_path);
    }

    Ok(status_output)
}

struct Commit {
    hash: String,
    author: String,
    date: String,
    message: String,
}

struct LogOutput(Vec<Commit>);

fn parse_log_output(
    log_output: &str,
) -> Result<LogOutput, GuitParseError> {


    todo!()
}

#[cfg(test)]
mod parse_commands_output_test {
    use super::*;

    const STATUS: &str = {
        r#" M compilateurs/projet_VS2017/ready.vcxproj
 M compilateurs/projet_VS2017/ready.vcxproj.filters
 M src/Ordonnanceur/TraitementNCTI.cpp
 M src/Ordonnanceur/TraitementNCTI.h
M  src/Sins/SinsService.cpp
M  src/Sins/SinsService.h"#
    };

    #[test]
    fn test_parse_status() {
        let expected = StatusOutput(
            vec![
                StatusPath { x: StatusTable::None, y: StatusTable::Modified, orig_path: None, path: Path("compilateurs/projet_VS2017/ready.vcxproj".to_owned()) },
                StatusPath { x: StatusTable::None, y: StatusTable::Modified, orig_path: None, path: Path("compilateurs/projet_VS2017/ready.vcxproj.filters".to_owned()) },
                StatusPath { x: StatusTable::None, y: StatusTable::Modified, orig_path: None, path: Path("src/Ordonnanceur/TraitementNCTI.cpp".to_owned()) },
                StatusPath { x: StatusTable::None, y: StatusTable::Modified, orig_path: None, path: Path("src/Ordonnanceur/TraitementNCTI.h".to_owned()) },

                StatusPath { x: StatusTable::Modified, y: StatusTable::None, orig_path: None, path: Path("src/Sins/SinsService.cpp".to_owned()) },
                StatusPath { x: StatusTable::Modified, y: StatusTable::None, orig_path: None, path: Path("src/Sins/SinsService.h".to_owned()) },
            ]
        );

        assert_eq!(parse_status_output(STATUS).unwrap(), expected);
    }
}
