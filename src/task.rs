use crate::config::CliError;
use std::{convert, fmt};

use jiff::{
    Unit, Zoned, civil,
    tz::{Offset, TimeZone},
};

struct Task {
    id: u32,
    desc: String,
    status: Status,
    created_at: Zoned,
    updated_at: Zoned,
}

impl Task {
    fn to_json(&self) -> String {
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
            self.status,
            self.created_at
                .round(Unit::Second)
                .unwrap()
                .strftime("%Y-%m-%dT%H:%M:%S%:z"),
            self.updated_at
                .round(Unit::Second)
                .unwrap()
                .strftime("%Y-%m-%dT%H:%M:%S%:z")
        )
    }

    fn tasks_to_json(tasks: Vec<Task>) -> String {
        let json_tasks: Vec<String> = tasks
            .iter()
            .map(|t| {
                t.to_json()
                    .lines()
                    .map(|line| format!("        {}", line))
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .collect();
        format!(
            r##"{{
    "tasks": [
{}
    ]
}}"##,
            json_tasks.join(",\n")
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Done,
    ToDo,
    InProgress,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Done => write!(f, "done"),
            Status::ToDo => write!(f, "todo"),
            Status::InProgress => write!(f, "in-progress"),
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

#[cfg(test)]
mod test {
    use jiff::tz::TimeZone;

    use super::*;

    #[test]
    fn test_tasks_to_json() {
        let dt = civil::date(2024, 2, 29).at(12, 51, 0, 0);
        let offset = Offset::from_hours(8).unwrap();
        let time = dt
            .to_zoned(TimeZone::fixed(offset))
            .expect("msg")
            .round(Unit::Second)
            .expect("msg");
        let tasks = vec![
            Task {
                id: 1,
                desc: "buy milk".to_string(),
                status: Status::InProgress,
                created_at: time.clone(),
                updated_at: time.clone(),
            },
            Task {
                id: 2,
                desc: "go home".to_string(),
                status: Status::ToDo,
                created_at: time.clone(),
                updated_at: time.clone(),
            },
        ];
        let actual = Task::tasks_to_json(tasks);
        let expected = r##"{
    "tasks": [
        {
            "id": 1,
            "desc": "buy milk",
            "status": "in-progress",
            "created_at": "2024-02-29T12:51:00+08:00",
            "updated_at": "2024-02-29T12:51:00+08:00"
        },
        {
            "id": 2,
            "desc": "go home",
            "status": "todo",
            "created_at": "2024-02-29T12:51:00+08:00",
            "updated_at": "2024-02-29T12:51:00+08:00"
        }
    ]
}"##;

        assert_eq!(actual, expected);
    }
}
