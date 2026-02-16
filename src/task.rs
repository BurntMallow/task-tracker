use crate::config::CliError;
use std::convert;

#[derive(Debug, PartialEq)]
pub enum Status {
    Done,
    ToDo,
    InProgress,
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
