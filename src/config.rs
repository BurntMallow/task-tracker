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
                Self::add_task(&mut tasks, desc, Zoned::now());
                Task::save(tasks)?;
                Ok(())
            }
            Command::Update { id, desc } => todo!(),
            Command::Delete(id) => todo!(),
            Command::MarkInProgress(id) => todo!(),
            Command::MarkDone(id) => todo!(),
            Command::List(status) => todo!(),
        }
    }

    fn add_task(tasks: &mut Vec<Task>, desc: String, now: Zoned) {
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
        let time = tasks.first().unwrap().created_at.clone();
        let new_time = get_new_time();
        Command::add_task(&mut tasks, desc.clone(), new_time.clone());
        let expected = vec![
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
                updated_at: time,
            },
            Task {
                id: 3,
                desc,
                status: Status::ToDo,
                created_at: new_time.clone(),
                updated_at: new_time,
            },
        ];
        assert_eq!(tasks, expected);
    }
}
