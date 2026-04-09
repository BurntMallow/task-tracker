use crate::config::CliError;
use std::{convert, error::Error, fmt, fs, io::ErrorKind};

use jiff::{Unit, Zoned};

#[derive(Debug, PartialEq)]
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
            self.created_at.round(Unit::Second).unwrap(),
            self.updated_at.round(Unit::Second).unwrap()
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

    pub fn load() -> Result<Vec<Self>, Box<dyn Error>> {
        let json: String = match fs::read_to_string("tasks.json") {
            Ok(content) => content,
            Err(ref e) if e.kind() == ErrorKind::NotFound => String::new(),
            Err(e) => return Err(Box::new(e)),
        };
        let tasks = Self::read_json(json)?;
        Ok(tasks)
    }

    fn read_json(json: String) -> Result<Vec<Self>, FileError> {
        let mut tasks: Vec<Self> = Vec::new();

        if json.is_empty() {
            return Ok(tasks);
        }
        let less_json = json
            .strip_prefix(
                r##"{
    "tasks": [
"##,
            )
            .ok_or(FileError::MissingTasks)?;

        let key_strs = [
            r##""id": "##,
            r##",
            "desc": ""##,
            r##"",
            "status": ""##,
            r##"",
            "created_at": ""##,
            r##"",
            "updated_at": ""##,
            r##"""##,
        ];

        for obj in less_json
            .trim_matches(|c| c == '[' || c == ']')
            .split("},\n        {")
        {
            let task = obj.trim_matches(|c| c == '{' || c == '}');

            let mut values = Vec::new();
            for key in key_strs.windows(2) {
                let value = Self::find_json_value(task, key[0], key[1])?;
                values.push(value.to_string());
            }

            tasks.push(Task {
                id: values[0].parse::<u32>().unwrap(),
                desc: values[1].clone(),
                status: Status::try_from(values[2].as_str()).unwrap(),
                created_at: values[3].parse().unwrap(),
                updated_at: values[4].parse().unwrap(),
            })
        }

        Ok(tasks)
    }

    // Returns value in between before key and after key in task json
    fn find_json_value(task: &str, before: &str, after: &str) -> Result<String, FileError> {
        let start: usize = task.find(before).ok_or(FileError::InvalidTask)? + before.len();
        let end = task.rfind(after).unwrap();
        Ok(task[start..end].to_string())
    }
}

#[derive(Debug, PartialEq)]
enum FileError {
    InvalidTask,
    MissingTasks,
}

impl Error for FileError {}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::InvalidTask => write!(f, "Inavlid Task"),
            FileError::MissingTasks => write!(f, "Missing Tasks"),
        }
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

    use jiff::civil;

    use super::*;

    static JSON_EXAMPLE: &str = r##"{
    "tasks": [
        {
            "id": 1,
            "desc": "buy milk",
            "status": "in-progress",
            "created_at": "2024-02-29T12:51:00+08:00[Asia/Manila]",
            "updated_at": "2024-02-29T12:51:00+08:00[Asia/Manila]"
        },
        {
            "id": 2,
            "desc": "go home",
            "status": "todo",
            "created_at": "2024-02-29T12:51:00+08:00[Asia/Manila]",
            "updated_at": "2024-02-29T12:51:00+08:00[Asia/Manila]"
        }
    ]
}"##;

    fn tasks_example() -> Vec<Task> {
        let time = civil::date(2024, 2, 29)
            .at(12, 51, 0, 0)
            .in_tz("Asia/Manila")
            .unwrap();
        vec![
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
        ]
    }

    #[test]
    fn test_tasks_to_json() {
        let tasks = tasks_example();
        let actual = Task::tasks_to_json(tasks);
        let expected = JSON_EXAMPLE;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_read_json() {
        let json = JSON_EXAMPLE.to_string();
        let actual = Task::read_json(json);
        let expected = tasks_example();
        assert_eq!(actual, Ok(expected));
    }
}
