# Task-Tracker CLI

A minimalist task manager built in Rust.

## Installation

### Download Release
1. Go to the [Latest Releases](https://github.com/BurntMallow/task-tracker/releases/latest) and download the archive (.tar.gz or .zip) for your operating system.
2. Unpack the archive to get the binary-name executable.
3. (For macOS/Linux) Open your terminal in the folder where you extracted the file and ensure it has permission to run by typing:
chmod +x task-tracker
4. (Optional) Move the binary to a directory in your system's PATH.

### Build from Source
```bash
git clone https://github.com/BurntMallow/task-tracker.git
cd task-tracker
cargo build --release
```

---

## Usage Guide

The application uses the structure: `task-tracker <action> [arguments]`

### Basic Commands
| Action | Example Command | Description |
| :--- | :--- | :--- |
| **Add** | `task-tracker add Buy groceries` | Create a new task |
| **Update** | `task-tracker update 1 Buy bread and milk` | Change a task description |
| **Delete** | `task-tracker delete 1` | Remove a task by ID |
| **Status** | `task-tracker mark-in-progress 1` | Set task to in-progress |
| **Status** | `task-tracker mark-done 1` | Set task to done |

### Listing and Filtering
| Filter | Command |
| :--- | :--- |
| **Show All** | `task-tracker list` |
| **To Do** | `task-tracker list todo` |
| **In Progress** | `task-tracker list in-progress` |
| **Done** | `task-tracker list done` |

---

## Technical Profile
* **Core:** Built using the Rust Standard Library for manual JSON parsing and CLI handling.
* **Time:** Uses [**Jiff**](https://github.com/BurntSushi/jiff) for `Zoned` timezone-aware timestamps.
* **Storage:** Local `tasks.json` persistence.

---

## Testing
```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

> Note: This project was developed as a solution to the [roadmap.sh Task Tracker project](https://roadmap.sh/projects/task-tracker).