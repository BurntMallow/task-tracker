use std::{
    error::Error,
    fs,
    io::Write,
    process::{Command, Stdio},
};

use jiff::Zoned;

#[test]
fn test_cli_flow() -> Result<(), Box<dyn Error>> {
    const ADD_USAGE: &str = "add <DESC>....          Create a new task";
    const UPDATE_USAGE: &str = "update <ID> <DESC>....  Change task description";
    const DELETE_USAGE: &str = "delete <ID>             Delete task";
    const MARK_IN_PROG_USAGE: &str = "mark-in-progress <ID>   Mark task status as in-progress";
    const MARK_DONE_USAGE: &str = "mark-done <ID>          Mark task status as done";
    const LIST_USAGE: &str = "list [STATUS]           List tasks (todo | done | in-progress)";

    let all_usage = format!(
        "\nUsage: task-tracker [OPTIONS]\n\nOptions:\n{}\n{}\n{}\n{}\n{}\n{}",
        ADD_USAGE, UPDATE_USAGE, DELETE_USAGE, MARK_IN_PROG_USAGE, MARK_DONE_USAGE, LIST_USAGE
    );

    let help_usage = format!("Manages your tasks\n{}", all_usage);

    let err_unknown_command = |unknown: &str| {
        format!(
            "Error: {} is not a command.\n{}",
            unknown,
            all_usage.clone()
        )
    };
    let err_missing_arg =
        |usage: &str| format!("Error: There are missing arguments.\n\nUsage: {}", usage);
    let err_invalid_id = |invalid_id: &str| format!("Error: {} is not a valid id.", invalid_id);
    let err_invalid_status = |unknown: &str| {
        format!(
            "Error: {} is not a valid status\n\nUsage: {}",
            unknown, LIST_USAGE
        )
    };

    let err_not_found = |id: u32| format!("Error: Task with ID {} not found", id);

    let ok_add = |id: u32| format!("Task added successfully (ID: {})", id);
    let ok_update = |id: u32| format!("Task {} updated successfully.", id);
    let ok_delete = |id: u32| format!("Task {} deleted successfully.", id);
    let ok_mark_in_prog = |id: u32| format!("Task {} marked as In-Progress.", id);
    let ok_mark_done = |id: u32| format!("Task {} marked as Done.", id);
    let ok_list = "--- All Tasks ---
[*] 1: hop (Done)
[*] 2: skip (Done)
[*] 3: jump (In-Progress)
[*] 5: takeoff (In-Progress)"
        .to_string();

    let ok_list_in_prog = "--- Tasks with Status: In-Progress ---
[*] 3: jump
[*] 5: takeoff"
        .to_string();

    let ok_list_done = "--- Tasks with Status: Done ---
[*] 1: hop
[*] 2: skip"
        .to_string();

    let scenarios = vec![
        ("", true, help_usage.clone()),
        ("list", true, "No Tasks found".to_string()),
        ("add", false, err_missing_arg(ADD_USAGE)),
        ("add hop", true, ok_add(1)),
        ("add skip", true, ok_add(2)),
        ("add bump", true, ok_add(3)),
        ("add float", true, ok_add(4)),
        ("add takeoff", true, ok_add(5)),
        ("update", false, err_missing_arg(UPDATE_USAGE)),
        ("update 3", false, err_missing_arg(UPDATE_USAGE)),
        ("update jump", false, err_invalid_id("jump")),
        ("update 3 jump", true, ok_update(3)),
        ("update 6 fly", false, err_not_found(6)),
        ("delete", false, err_missing_arg(DELETE_USAGE)),
        ("delete 6", false, err_not_found(6)),
        ("delete five", false, err_invalid_id("five")),
        ("delete 4", true, ok_delete(4)),
        ("mark-in-progress 2", true, ok_mark_in_prog(2)),
        ("mark-in-progress 3", true, ok_mark_in_prog(3)),
        ("mark-done 1", true, ok_mark_done(1)),
        ("mark-done 2", true, ok_mark_done(2)),
        ("mark-done 5", true, ok_mark_done(5)),
        ("mark-in-progress 5", true, ok_mark_in_prog(5)),
        ("mark-done 4", false, err_not_found(4)),
        ("mark-in-progress six", false, err_invalid_id("six")),
        ("mark-todo 1", false, err_unknown_command("mark-todo")),
        ("list", true, ok_list),
        ("list 1", false, err_invalid_status("1")),
        (
            "list todo",
            true,
            "No Tasks with Status Todo found".to_string(),
        ),
        ("list in-progress", true, ok_list_in_prog),
        ("list done", true, ok_list_done),
        ("list all", false, err_invalid_status("all")),
    ];

    let expected_json = vec![
        r##"{"##,
        r##"    "tasks": ["##,
        r##"        {"##,
        r##"            "id": 1,"##,
        r##"            "desc": "hop","##,
        r##"            "status": "done","##,
        r##"            "created_at": "*","##,
        r##"            "updated_at": "*""##,
        r##"        },"##,
        r##"        {"##,
        r##"            "id": 2,"##,
        r##"            "desc": "skip","##,
        r##"            "status": "done","##,
        r##"            "created_at": "*","##,
        r##"            "updated_at": "*""##,
        r##"        },"##,
        r##"        {"##,
        r##"            "id": 3,"##,
        r##"            "desc": "jump","##,
        r##"            "status": "in-progress","##,
        r##"            "created_at": "*","##,
        r##"            "updated_at": "*""##,
        r##"        },"##,
        r##"        {"##,
        r##"            "id": 5,"##,
        r##"            "desc": "takeoff","##,
        r##"            "status": "in-progress","##,
        r##"            "created_at": "*","##,
        r##"            "updated_at": "*""##,
        r##"        }"##,
        r##"    ]"##,
        r##"}"##,
    ];

    let bin_path = env!("CARGO_BIN_EXE_task-tracker");

    let mut file_path = std::env::current_dir()?;
    file_path.push("test_tasks.json");
    let file_path_str = file_path.to_str().expect("Valid UTF-8 path");

    let mut file_path_temp = std::env::current_dir()?;
    file_path_temp.push("test_tasks.json.tmp");
    let file_path_temp_str = file_path_temp.to_str().expect("Valid UTF-8 path");

    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }

    for (input, should_succeed, expected_str) in scenarios {
        let args: Vec<&str> = input.split_whitespace().collect();

        let mut child = Command::new(bin_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("TASKS_FILE_PATH", file_path_str)
            .env("TASKS_FILE_PATH_TEMP", file_path_temp_str)
            .spawn()
            .expect("Failed to spawn process");

        {
            let mut stdin = child.stdin.take().expect("Failed to open stdin");
            if let Err(e) = stdin.write_all(input.as_bytes()) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("Failed to write to stdin: {}", e);
            }
        }

        let output = child.wait_with_output().expect("Failed to read output");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_ui = format!("{}{}", stdout, stderr);

        assert_eq!(
            output.status.success(),
            should_succeed,
            "Unexpected exit status for input: {}",
            input
        );

        let mut ui_cursor = 0;
        for fragment in expected_str.split('*') {
            if let Some(pos) = combined_ui[ui_cursor..].find(fragment) {
                ui_cursor += pos + fragment.len();
            } else {
                panic!(
                    "\n[UI MATCH FAILED]\nCommand: {}\nMissing Fragment: '{}'\nActual Output:\n{}",
                    input, fragment, combined_ui
                );
            }
        }
    }

    let file_content = fs::read_to_string(file_path_str)?;
    let mut json_cursor = 0;

    for line_template in expected_json {
        for fragment in line_template.split('*') {
            if let Some(pos) = file_content[json_cursor..].find(fragment) {
                json_cursor += pos + fragment.len();
            } else {
                panic!(
                    "\n[JSON FILE MATCH FAILED]\nMissing Fragment: '{}'\nFull File Content:\n{}",
                    fragment, file_content
                );
            }
        }
    }

    let mut timestamps = Vec::new();
    let pattern = "_at\": \"";

    for (i, _) in file_content.match_indices(pattern) {
        let start = i + pattern.len();
        if let Some(end_offset) = file_content[start..].find('"') {
            let ts_str = &file_content[start..start + end_offset];
            let zoned = ts_str
                .parse::<Zoned>()
                .expect("JSON should contain valid Jiff ISO 8601 strings");
            timestamps.push(zoned);
        }
    }

    assert!(
        timestamps[0] <= timestamps[2],
        "Task 1 Created At should be before or same second as Task 2 Created At"
    );
    assert!(
        timestamps[2] <= timestamps[4],
        "Task 2 Created At should be before or same second as Task 3 Created At"
    );
    assert!(
        timestamps[4] <= timestamps[6],
        "Task 3 Created At should be before or same second as Task 5 Created At"
    );
    assert!(
        timestamps[6] <= timestamps[5],
        "Task 5 Created At should be before or same second as Task 3 Updated At"
    );
    assert!(
        timestamps[5] <= timestamps[1],
        "Task 3 Updated At should be before or same second as Task 1 Updated At"
    );
    assert!(
        timestamps[1] <= timestamps[3],
        "Task 1 Updated At should be before or same second as Task 2 Updated At"
    );
    assert!(
        timestamps[3] <= timestamps[7],
        "Task 2 Updated At should be before or same second as Task 5 Updated At"
    );

    fs::remove_file(&file_path)?;

    Ok(())
}
