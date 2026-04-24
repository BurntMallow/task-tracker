use crate::config::CliError;
use std::{convert, fmt};

use jiff::{Unit, Zoned};

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Done,
    ToDo,
    InProgress,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Task {
    pub id: u32,
    pub desc: String,
    pub status: Status,
    pub created_at: Zoned,
    pub updated_at: Zoned,
}

impl Task {
    pub fn to_json(&self) -> String {
        // Raw string literal used to maintain exact indentation for the manual JSON parser
        format!(
            r##"{{
    "id": {},
    "desc": "{}",
    "status": "{}",
    "created_at": "{}",
    "updated_at": "{}"
}}"##,
            self.id,
            self.desc,
            self.status.as_str(),
            // Rounding to seconds removes nanosecond "noise" while keeping
            // the offset/zone required for Zoned::parse().unwrap() to work later.
            self.created_at.round(Unit::Second).unwrap(),
            self.updated_at.round(Unit::Second).unwrap()
        )
    }
}

impl Status {
    // Machine-friendly string representation for JSON keys and CLI arguments
    fn as_str(&self) -> &'static str {
        match self {
            Self::ToDo => "todo",
            Self::InProgress => "in-progress",
            Self::Done => "done",
        }
    }
}

impl convert::TryFrom<&str> for Status {
    type Error = CliError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "done" => Ok(Status::Done),
            "todo" => Ok(Status::ToDo),
            "in-progress" => Ok(Status::InProgress),
            _ => Err(CliError::InvalidStatus(value.to_string())),
        }
    }
}

impl fmt::Display for Status {
    // User-friendly string representation for terminal UI output
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Done => write!(f, "Done"),
            Status::ToDo => write!(f, "Todo"),
            Status::InProgress => write!(f, "In-Progress"),
        }
    }
}

#[cfg(test)]
pub(crate) fn tasks_example() -> Vec<Task> {
    use jiff::civil;

    let time = civil::date(2024, 2, 29)
        .at(12, 51, 0, 0)
        .in_tz("Asia/Manila")
        .unwrap();
    vec![
        Task {
            id: 1,
            desc: "take a break".to_string(),
            status: Status::Done,
            created_at: time.clone(),
            updated_at: time.clone(),
        },
        Task {
            id: 2,
            desc: "buy milk".to_string(),
            status: Status::InProgress,
            created_at: time.clone(),
            updated_at: time.clone(),
        },
        Task {
            id: 3,
            desc: "go home".to_string(),
            status: Status::ToDo,
            created_at: time.clone(),
            updated_at: time,
        },
    ]
}
