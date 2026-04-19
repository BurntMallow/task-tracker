use std::error::Error;
use std::{convert, fmt};

use jiff::Zoned;

use crate::task::{Status, Task};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub command: Command,
}

impl Config {
    pub fn build(args: Vec<String>) -> Result<Config, CliError> {
        let mut iter = args.into_iter().skip(1);
        let cmd = iter.next().unwrap_or_default();
        let kind = CommandKind::try_from(cmd.as_str())?;

        match kind {
            CommandKind::Help => Ok(Config {
                command: Command::Help,
            }),
            CommandKind::Add => {
                let desc = Self::next_desc(&mut iter, kind)?;
                Ok(Config {
                    command: Command::Add(desc),
                })
            }
            CommandKind::Update => {
                let id = Self::next_id(&mut iter, kind.clone())?;
                let desc = Self::next_desc(&mut iter, kind)?;
                Ok(Config {
                    command: Command::Update { id, desc },
                })
            }
            CommandKind::Delete => {
                let id = Self::next_id(&mut iter, kind)?;
                Ok(Config {
                    command: Command::Delete(id),
                })
            }
            CommandKind::MarkInProgress => {
                let id = Self::next_id(&mut iter, kind)?;
                Ok(Config {
                    command: Command::MarkInProgress(id),
                })
            }
            CommandKind::MarkDone => {
                let id = Self::next_id(&mut iter, kind)?;
                Ok(Config {
                    command: Command::MarkDone(id),
                })
            }
            CommandKind::List => match iter.next() {
                None => Ok(Config {
                    command: Command::List(None),
                }),
                Some(s) => {
                    let status = Status::try_from(s.as_str())?;
                    Ok(Config {
                        command: Command::List(Some(status)),
                    })
                }
            },
        }
    }

    fn next_desc(
        iter: &mut impl Iterator<Item = String>,
        kind: CommandKind,
    ) -> Result<String, CliError> {
        iter.map(|s| s.to_string())
            .reduce(|mut acc, item| {
                acc.push(' ');
                acc.push_str(&item);
                acc
            })
            .ok_or(CliError::MissingArgument(kind))
    }

    fn next_id(
        iter: &mut impl Iterator<Item = String>,
        kind: CommandKind,
    ) -> Result<u32, CliError> {
        let id_str = iter.next().ok_or(CliError::MissingArgument(kind))?;
        let id_u32 = id_str
            .parse::<u32>()
            .map_err(|_| CliError::InvalidId(id_str))?;
        Ok(id_u32)
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Add(String),
    Update { id: u32, desc: String },
    Delete(u32),
    MarkInProgress(u32),
    MarkDone(u32),
    List(Option<Status>),
}

impl Command {
    pub fn execute(self) -> Result<(), Box<dyn Error>> {
        match self {
            Command::Help => {
                println!("Manages your tasks\n{}", show_all_usage());
                Ok(())
            }
            Command::Add(desc) => {
                let mut tasks = Task::load()?;
                let id = Self::add_task(&mut tasks, desc, Zoned::now());
                Task::save(tasks)?;
                println!("Task added successfully (ID: {})", id);
                Ok(())
            }
            Command::Update { id, desc } => {
                let mut tasks = Task::load()?;
                Self::update_task(&mut tasks, id, desc, Zoned::now())?;
                Task::save(tasks)?;
                println!("Task {} updated successfully.", id);
                Ok(())
            }
            Command::Delete(id) => {
                let mut tasks = Task::load()?;
                Self::delete_task(&mut tasks, id)?;
                Task::save(tasks)?;
                println!("Task {} deleted successfully.", id);
                Ok(())
            }
            Command::MarkInProgress(id) => {
                let mut tasks = Task::load()?;
                Self::mark_task(&mut tasks, id, Status::InProgress, Zoned::now())?;
                Task::save(tasks)?;
                println!("Task {} marked as {}.", id, Status::InProgress);
                Ok(())
            }
            Command::MarkDone(id) => {
                let mut tasks = Task::load()?;
                Self::mark_task(&mut tasks, id, Status::Done, Zoned::now())?;
                Task::save(tasks)?;
                println!("Task {} marked as {}.", id, Status::Done);
                Ok(())
            }
            Command::List(status) => {
                let tasks = Task::load()?;
                Self::list_tasks(tasks, status);
                Ok(())
            }
        }
    }

    fn add_task(tasks: &mut Vec<Task>, desc: String, now: Zoned) -> u32 {
        let id = match tasks.last() {
            Some(task) => task.id + 1,
            None => 1,
        };
        tasks.push(Task {
            id,
            desc: desc.to_string(),
            status: Status::ToDo,
            created_at: now.clone(),
            updated_at: now,
        });

        id
    }

    fn update_task(
        tasks: &mut [Task],
        id: u32,
        new_desc: String,
        time: Zoned,
    ) -> Result<(), CommandError> {
        let task = Self::find_task(tasks, id)?;
        task.desc = new_desc;
        task.updated_at = time;
        Ok(())
    }

    fn delete_task(tasks: &mut Vec<Task>, id: u32) -> Result<(), CommandError> {
        Self::find_task(tasks, id)?;
        tasks.retain(|t| t.id != id);
        Ok(())
    }

    fn mark_task(
        tasks: &mut [Task],
        id: u32,
        status: Status,
        time: Zoned,
    ) -> Result<(), CommandError> {
        let task = Self::find_task(tasks, id)?;
        task.status = status;
        task.updated_at = time;
        Ok(())
    }

    fn list_tasks(mut tasks: Vec<Task>, status: Option<Status>) -> Vec<Task> {
        tasks.retain(|t| status.as_ref().is_none_or(|s| t.status == *s));

        if tasks.is_empty() {
            if let Some(s) = status {
                println!("No Tasks with Status {} found", s);
            } else {
                println!("No Tasks found");
            }
        } else if let Some(s) = status {
            println!("--- Tasks with Status: {} ---", s);
            for task in tasks.iter().filter(|t| t.status == s) {
                println!(
                    "[{}] {}: {}",
                    task.created_at.strftime("%Y-%m-%d %H:%M"),
                    task.id,
                    task.desc
                );
            }
        } else {
            println!("--- All Tasks ---");
            for task in &tasks {
                println!(
                    "[{}] {}: {} ({})",
                    task.created_at.strftime("%Y-%m-%d %H:%M"),
                    task.id,
                    task.desc,
                    task.status
                );
            }
        }

        tasks
    }

    fn find_task(tasks: &mut [Task], id: u32) -> Result<&mut Task, CommandError> {
        tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(CommandError::NotFound(id))
    }
}

#[derive(Debug, PartialEq)]
enum CommandError {
    NotFound(u32),
}

impl Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::NotFound(id) => write!(f, "Task with ID {id} not found"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandKind {
    Help,
    Add,
    Update,
    Delete,
    MarkInProgress,
    MarkDone,
    List,
}

impl convert::TryFrom<&str> for CommandKind {
    type Error = CliError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "" => Ok(CommandKind::Help),
            "add" => Ok(CommandKind::Add),
            "update" => Ok(CommandKind::Update),
            "delete" => Ok(CommandKind::Delete),
            "mark-in-progress" => Ok(CommandKind::MarkInProgress),
            "mark-done" => Ok(CommandKind::MarkDone),
            "list" => Ok(CommandKind::List),
            _ => Err(CliError::UnknownCommand(value.to_string())),
        }
    }
}

impl CommandKind {
    fn usage(&self) -> &str {
        match self {
            CommandKind::Help => "",
            CommandKind::Add => "add <DESC>....          Create a new task",
            CommandKind::Update => "update <ID> <DESC>....  Change task description",
            CommandKind::Delete => "delete <ID>             Delete task",
            CommandKind::MarkInProgress => {
                "mark-in-progress <ID>   Mark task status as in-progress"
            }
            CommandKind::MarkDone => "mark-done <ID>          Mark task status as done",
            CommandKind::List => "list [STATUS]           List tasks (todo | done | in-progress)",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CliError {
    UnknownCommand(String),
    MissingArgument(CommandKind),
    InvalidId(String),
    InvalidStatus(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::UnknownCommand(s) => {
                write!(f, "Error: {} is not a command.\n{}", s, show_all_usage())
            }
            CliError::MissingArgument(cmd) => write!(
                f,
                "Error: There are missing arguments.\n\nUsage: {}",
                cmd.usage()
            ),
            CliError::InvalidId(s) => write!(f, "Error: {} is not a valid id.", s),
            CliError::InvalidStatus(s) => write!(
                f,
                "Error: {} is not a valid status\n\nUsage: {}",
                s,
                CommandKind::List.usage()
            ),
        }
    }
}

fn show_all_usage() -> String {
    format!(
        "\nUsage: todo [OPTIONS]\n\nOptions:\n{}\n{}\n{}\n{}\n{}\n{}",
        CommandKind::Add.usage(),
        CommandKind::Update.usage(),
        CommandKind::Delete.usage(),
        CommandKind::MarkInProgress.usage(),
        CommandKind::MarkDone.usage(),
        CommandKind::List.usage(),
    )
}

#[cfg(test)]
mod tests {
    use jiff::civil;

    use super::*;
    use crate::task::tasks_example;

    fn get_new_time() -> Zoned {
        civil::date(2025, 12, 25)
            .at(11, 11, 11, 0)
            .in_tz("Asia/Manila")
            .unwrap()
    }

    #[test]
    fn test_build() {
        let cases = vec![
            (
                vec!["task"],
                Ok(Config {
                    command: Command::Help,
                }),
            ),
            (
                vec!["task", "add", "buy", "milk"],
                Ok(Config {
                    command: Command::Add("buy milk".to_string()),
                }),
            ),
            (
                vec!["task", "add"],
                Err(CliError::MissingArgument(CommandKind::Add)),
            ),
            (
                vec!["task", "update", "1", "go", "home"],
                Ok(Config {
                    command: Command::Update {
                        id: 1,
                        desc: "go home".to_string(),
                    },
                }),
            ),
            (
                vec!["task", "update"],
                Err(CliError::MissingArgument(CommandKind::Update)),
            ),
            (
                vec!["task", "update", "1"],
                Err(CliError::MissingArgument(CommandKind::Update)),
            ),
            (
                vec!["task", "update", "one", "go", "home"],
                Err(CliError::InvalidId("one".to_string())),
            ),
            (
                vec!["task", "delete", "1"],
                Ok(Config {
                    command: Command::Delete(1),
                }),
            ),
            (
                vec!["task", "delete"],
                Err(CliError::MissingArgument(CommandKind::Delete)),
            ),
            (
                vec!["task", "delete", "one"],
                Err(CliError::InvalidId("one".to_string())),
            ),
            (
                vec!["task", "mark-in-progress", "1"],
                Ok(Config {
                    command: Command::MarkInProgress(1),
                }),
            ),
            (
                vec!["task", "mark-in-progress"],
                Err(CliError::MissingArgument(CommandKind::MarkInProgress)),
            ),
            (
                vec!["task", "mark-in-progress", "one"],
                Err(CliError::InvalidId("one".to_string())),
            ),
            (
                vec!["task", "mark-done", "1"],
                Ok(Config {
                    command: Command::MarkDone(1),
                }),
            ),
            (
                vec!["task", "mark-done"],
                Err(CliError::MissingArgument(CommandKind::MarkDone)),
            ),
            (
                vec!["task", "mark-done", "one"],
                Err(CliError::InvalidId("one".to_string())),
            ),
            (
                vec!["task", "list"],
                Ok(Config {
                    command: Command::List(None),
                }),
            ),
            (
                vec!["task", "list", "done"],
                Ok(Config {
                    command: Command::List(Some(Status::Done)),
                }),
            ),
            (
                vec!["task", "list", "todo"],
                Ok(Config {
                    command: Command::List(Some(Status::ToDo)),
                }),
            ),
            (
                vec!["task", "list", "in-progress"],
                Ok(Config {
                    command: Command::List(Some(Status::InProgress)),
                }),
            ),
            (
                vec!["task", "list", "maybe"],
                Err(CliError::InvalidStatus("maybe".to_string())),
            ),
            (
                vec!["task", "discombobulate"],
                Err(CliError::UnknownCommand("discombobulate".to_string())),
            ),
        ];
        for (input_strs, expected) in cases {
            let input: Vec<String> = input_strs.into_iter().map(String::from).collect();
            let actual = Config::build(input);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_add_task() {
        let mut tasks = tasks_example();
        let desc = "example".to_string();
        let new_time = get_new_time();
        let expected = vec![
            tasks[0].clone(),
            tasks[1].clone(),
            tasks[2].clone(),
            Task {
                id: 4,
                desc: desc.clone(),
                status: Status::ToDo,
                created_at: new_time.clone(),
                updated_at: new_time.clone(),
            },
        ];

        Command::add_task(&mut tasks, desc, new_time);
        assert_eq!(
            tasks, expected,
            "The task list should contain the original tasks plus the newly added one at the end"
        );
    }

    #[test]
    fn test_update_task() {
        let mut tasks = tasks_example();
        let new_desc = "example".to_string();
        let new_time = get_new_time();
        let expected = vec![
            tasks[0].clone(),
            Task {
                id: tasks[1].id,
                desc: new_desc.clone(),
                status: tasks[1].status.clone(),
                created_at: tasks[1].created_at.clone(),
                updated_at: new_time.clone(),
            },
            tasks[2].clone(),
        ];

        let err = Command::update_task(&mut tasks, 999, new_desc.clone(), new_time.clone());
        assert_eq!(
            err,
            Err(CommandError::NotFound(999)),
            "Should return NotFound for non-existent ID"
        );
        assert_eq!(
            tasks,
            tasks_example(),
            "Task list should remain unchanged after a failed update"
        );

        let ok = Command::update_task(&mut tasks, 2, new_desc, new_time);

        assert!(ok.is_ok(), "Update should return Ok for a valid ID");
        assert_eq!(
            tasks, expected,
            "Task 2 should have updated description/timestamp while Task 1 remains untouched"
        );
    }

    #[test]
    fn test_delete_task() {
        let mut tasks = tasks_example();
        let expected = vec![tasks[1].clone(), tasks[2].clone()];

        let err = Command::delete_task(&mut tasks, 999);
        assert_eq!(
            err,
            Err(CommandError::NotFound(999)),
            "Should fail when deleting non-existent ID"
        );
        assert_eq!(
            tasks,
            tasks_example(),
            "Should not change tasks when delete errs"
        );

        let ok = Command::delete_task(&mut tasks, 1);
        assert!(ok.is_ok(), "Should successfully delete ID 1");
        assert_eq!(
            tasks, expected,
            "Task 1 should have deleted from the list while Task 2 remains untouched"
        );
    }

    #[test]
    fn test_mark_task() {
        let mut tasks = tasks_example();
        let new_time = get_new_time();
        let expected = vec![
            Task {
                id: tasks[0].id,
                desc: tasks[0].desc.clone(),
                status: Status::ToDo,
                created_at: tasks[0].created_at.clone(),
                updated_at: new_time.clone(),
            },
            Task {
                id: tasks[1].id,
                desc: tasks[1].desc.clone(),
                status: Status::Done,
                created_at: tasks[1].created_at.clone(),
                updated_at: new_time.clone(),
            },
            Task {
                id: tasks[2].id,
                desc: tasks[2].desc.clone(),
                status: Status::InProgress,
                created_at: tasks[2].created_at.clone(),
                updated_at: new_time.clone(),
            },
        ];

        let err = Command::mark_task(&mut tasks, 999, Status::Done, new_time.clone());
        assert_eq!(
            err,
            Err(CommandError::NotFound(999)),
            "Should return NotFound for non-existent ID"
        );
        assert_eq!(
            tasks,
            tasks_example(),
            "Task list should remain unchanged after a failed status update"
        );

        let todo_ok = Command::mark_task(&mut tasks, 1, Status::ToDo, new_time.clone());
        let done_ok = Command::mark_task(&mut tasks, 2, Status::Done, new_time.clone());
        let in_progress_ok = Command::mark_task(&mut tasks, 3, Status::InProgress, new_time);

        assert!(
            todo_ok.is_ok(),
            "Mark Status ToDo should return Ok for a valid ID"
        );
        assert!(
            done_ok.is_ok(),
            "Mark Status Done should return Ok for a valid ID"
        );
        assert!(
            in_progress_ok.is_ok(),
            "Mark Status InProgress should return Ok for a valid ID"
        );
        assert_eq!(
            tasks, expected,
            "Task 1 should be ToDo, Task 2 should be Done, and Task3 should be InProgress with fresh timestamps"
        );
    }

    #[test]
    fn test_list_task() {
        let tasks = tasks_example();

        let empty_list = Command::list_tasks(vec![], None);
        assert!(
            empty_list.is_empty(),
            "Shoudl return empty for an empty input"
        );

        let all_list = Command::list_tasks(tasks.clone(), None);
        assert_eq!(
            all_list,
            tasks.clone(),
            "Should return all tasks when given no status"
        );

        let done_list = Command::list_tasks(tasks.clone(), Some(Status::Done));
        let expected_done_list = vec![Task {
            id: tasks[0].id,
            desc: tasks[0].desc.clone(),
            status: tasks[0].status.clone(),
            created_at: tasks[0].created_at.clone(),
            updated_at: tasks[0].updated_at.clone(),
        }];
        assert_eq!(
            done_list, expected_done_list,
            "Should only return tasks with Done status"
        );

        let in_progress_list = Command::list_tasks(tasks.clone(), Some(Status::InProgress));
        let expected_in_progress_list = vec![Task {
            id: tasks[1].id,
            desc: tasks[1].desc.clone(),
            status: tasks[1].status.clone(),
            created_at: tasks[1].created_at.clone(),
            updated_at: tasks[1].updated_at.clone(),
        }];
        assert_eq!(
            in_progress_list, expected_in_progress_list,
            "Should only return tasks with InProgress status"
        );

        let todo_list = Command::list_tasks(tasks.clone(), Some(Status::ToDo));
        let expected_todo_list = vec![Task {
            id: tasks[2].id,
            desc: tasks[2].desc.clone(),
            status: tasks[2].status.clone(),
            created_at: tasks[2].created_at.clone(),
            updated_at: tasks[2].updated_at.clone(),
        }];
        assert_eq!(
            todo_list, expected_todo_list,
            "Should only return tasks with ToDo status"
        );

        let not_found_list = Command::list_tasks(todo_list, Some(Status::Done));
        assert!(
            not_found_list.is_empty(),
            "Should return empty when no tasks with given status is found"
        )
    }

    #[test]
    fn test_find_task() {
        let mut tasks = tasks_example();

        let found = Command::find_task(&mut tasks, 1);
        assert!(found.is_ok(), "Task with ID 1 should be found");
        assert_eq!(found.unwrap().id, 1);

        let not_found = Command::find_task(&mut tasks, 999);
        assert_eq!(
            not_found.err(),
            Some(CommandError::NotFound(999)),
            "Should return NotFound for ID 999"
        );
    }
}
