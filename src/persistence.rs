use std::{env::var, error::Error, fmt, fs, io::ErrorKind};

use crate::task::{Status, Task};

pub fn load() -> Result<Vec<Task>, Box<dyn Error>> {
    let file_path = var("TASKS_FILE_PATH").unwrap_or_else(|_| "tasks.json".to_string());
    let json: String = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(ref e) if e.kind() == ErrorKind::NotFound => String::new(),
        Err(e) => return Err(Box::new(e)),
    };
    let tasks = read_json(json)?;
    Ok(tasks)
}

pub fn save(tasks: Vec<Task>) -> Result<(), Box<dyn Error>> {
    let path = var("TASKS_FILE_PATH").unwrap_or_else(|_| "tasks.json".to_string());
    let temp_path = var("TASKS_FILE_PATH_TEMP").unwrap_or_else(|_| "tasks.json.tmp".to_string());
    let contents = tasks_to_json(tasks);
    fs::write(temp_path.clone(), contents.as_bytes())?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn read_json(json: String) -> Result<Vec<Task>, FileError> {
    let mut tasks: Vec<Task> = Vec::new();

    if json.is_empty() {
        return Ok(tasks);
    }
    let less_json = json
        .strip_prefix(
            r##"{
    "tasks": [
"##,
        )
        .ok_or(FileError::MissingTasks)?
        .strip_suffix(
            r##"]
}"##,
        )
        .ok_or(FileError::MissingTasks)?;

    if less_json.trim().is_empty() {
        return Ok(tasks);
    }

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
            let value = find_json_value(task, key[0], key[1])?;
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

#[derive(Debug, PartialEq)]
pub enum FileError {
    InvalidTask,
    MissingTasks,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::InvalidTask => write!(f, "Error: Inavlid Task found in json file"),
            FileError::MissingTasks => write!(f, "Error: Tasks array is missing from json file"),
        }
    }
}

impl Error for FileError {}

#[cfg(test)]
mod test {

    use crate::task::tasks_example;

    use super::*;

    static JSON_EXAMPLE: &str = r##"{
    "tasks": [
        {
            "id": 1,
            "desc": "take a break",
            "status": "done",
            "created_at": "2024-02-29T12:51:00+08:00[Asia/Manila]",
            "updated_at": "2024-02-29T12:51:00+08:00[Asia/Manila]"
        },
        {
            "id": 2,
            "desc": "buy milk",
            "status": "in-progress",
            "created_at": "2024-02-29T12:51:00+08:00[Asia/Manila]",
            "updated_at": "2024-02-29T12:51:00+08:00[Asia/Manila]"
        },
        {
            "id": 3,
            "desc": "go home",
            "status": "todo",
            "created_at": "2024-02-29T12:51:00+08:00[Asia/Manila]",
            "updated_at": "2024-02-29T12:51:00+08:00[Asia/Manila]"
        }
    ]
}"##;

    #[test]
    fn test_tasks_to_json() {
        let tasks = tasks_example();
        let actual = tasks_to_json(tasks);
        let expected = JSON_EXAMPLE;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_read_json() {
        let json = JSON_EXAMPLE.to_string();
        let actual = read_json(json);
        let expected = tasks_example();
        assert_eq!(actual, Ok(expected));
    }
}
