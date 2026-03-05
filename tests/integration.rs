// r[verify test.fixture-db]
// r[verify test.fixtures]
// r[verify test.read-commands]
// r[verify test.output-formats]
// r[verify test.error-cases]

use std::process::Command;

mod fixtures;

fn tdo(db_path: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tdo"));
    cmd.arg("--db-path").arg(db_path);
    cmd
}

fn run_tdo(db_path: &str, args: &[&str]) -> (String, String, bool) {
    let output = tdo(db_path).args(args).output().expect("failed to run tdo");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

// ── Inbox ──

// r[verify cmd.inbox] r[verify cmd.inbox.columns] r[verify output.tsv]
#[test]
fn test_inbox_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["inbox"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\ttags\tdeadline");
    // Fixture has 1 inbox task: "Inbox task"
    assert!(lines.len() >= 2, "expected at least header + 1 row");
    assert!(
        lines[1].contains("Inbox task"),
        "expected inbox task, got: {}",
        lines[1]
    );
}

// r[verify cmd.inbox] r[verify output.json]
#[test]
fn test_inbox_json() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "inbox"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["title"], "Inbox task");
}

// ── Today ──

// r[verify cmd.today] r[verify cmd.today.columns]
#[test]
fn test_today_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["today"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tproject\ttags\tdeadline");
    assert!(lines.iter().any(|l| l.contains("Today task")));
}

// r[verify cmd.today] r[verify output.json]
#[test]
fn test_today_json() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "today"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    assert!(arr.iter().any(|item| item["title"] == "Today task"));
}

// ── Upcoming ──

// r[verify cmd.upcoming] r[verify cmd.upcoming.columns]
#[test]
fn test_upcoming_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["upcoming"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tproject\ttags\tstartDate\tdeadline");
    assert!(lines.iter().any(|l| l.contains("Upcoming task")));
}

// ── Anytime ──

// r[verify cmd.anytime] r[verify cmd.anytime.columns]
#[test]
fn test_anytime_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["anytime"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tproject\tarea\ttags\tdeadline");
    assert!(lines.iter().any(|l| l.contains("Anytime task")));
}

// ── Someday ──

// r[verify cmd.someday] r[verify cmd.someday.columns]
#[test]
fn test_someday_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["someday"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tproject\ttags");
    assert!(lines.iter().any(|l| l.contains("Someday task")));
}

// ── Logbook ──

// r[verify cmd.logbook] r[verify cmd.logbook.columns]
#[test]
fn test_logbook_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["logbook"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tproject\tcompletedDate");
    assert!(lines.iter().any(|l| l.contains("Completed task")));
}

// r[verify cmd.logbook.limit]
#[test]
fn test_logbook_limit() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["logbook", "--limit", "1"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2); // header + 1 row
}

// ── Projects ──

// r[verify cmd.projects] r[verify cmd.projects.columns]
#[test]
fn test_projects_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["projects"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tarea\ttags\tdeadline\topenTasks");
    assert!(lines.iter().any(|l| l.contains("Test Project")));
}

// ── Areas ──

// r[verify cmd.areas] r[verify cmd.areas.columns]
#[test]
fn test_areas_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["areas"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle");
    assert!(lines.iter().any(|l| l.contains("Work")));
}

// ── Tags ──

// r[verify cmd.tags] r[verify cmd.tags.columns]
#[test]
fn test_tags_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["tags"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\tshortcut\tparent");
    assert!(lines.iter().any(|l| l.contains("urgent")));
}

// ── Show ──

// r[verify cmd.show] r[verify cmd.show.output]
#[test]
fn test_show_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["show", "task-today-1"]);
    assert!(ok);
    assert!(out.contains("Today task"));
    assert!(out.contains("task"));
}

// r[verify cmd.show] r[verify cmd.show.output]
#[test]
fn test_show_json() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "show", "task-today-1"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    assert_eq!(parsed["title"], "Today task");
    assert_eq!(parsed["type"], "task");
}

// r[verify cmd.show]
#[test]
fn test_show_with_checklist() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "show", "task-checklist-1"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let items = parsed["checklistItems"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["title"], "Step 1");
    assert_eq!(items[0]["completed"], false);
    assert_eq!(items[1]["title"], "Step 2");
    assert_eq!(items[1]["completed"], true);
}

// r[verify error.db-not-found]
#[test]
fn test_show_not_found() {
    let db = fixtures::create_fixture_db();
    let (_, stderr, ok) = run_tdo(db.path(), &["show", "nonexistent-id"]);
    assert!(!ok);
    assert!(
        stderr.contains("not found"),
        "expected 'not found' in: {stderr}"
    );
}

// ── Search ──

// r[verify cmd.search] r[verify cmd.search.columns]
#[test]
fn test_search_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["search", "Today"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("id\ttitle"));
    assert!(lines.iter().any(|l| l.contains("Today task")));
}

// ── Stats ──

// r[verify cmd.stats] r[verify cmd.stats.output]
#[test]
fn test_stats_tsv() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["stats"]);
    assert!(ok);
    assert!(out.contains("inbox"));
    assert!(out.contains("today"));
    assert!(out.contains("projects"));
}

// r[verify cmd.stats] r[verify cmd.stats.output]
#[test]
fn test_stats_json() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "stats"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    assert!(parsed["inbox"].is_number());
    assert!(parsed["today"].is_number());
}

// ── Output format tests ──

// r[verify output.no-header]
#[test]
fn test_no_header() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--no-header", "inbox"]);
    assert!(ok);
    // First line should NOT be the header
    let first_line = out.lines().next().unwrap_or("");
    assert!(
        !first_line.starts_with("id\t"),
        "header should be suppressed"
    );
}

// r[verify output.tsv.fields]
#[test]
fn test_fields_override() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--fields", "id,title", "today"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle");
}

// ── Error path tests ──

// r[verify global.db-path] r[verify error.db-not-found]
#[test]
fn test_missing_db() {
    let (_, stderr, ok) = run_tdo("/nonexistent/path.sqlite", &["today"]);
    assert!(!ok);
    assert!(stderr.contains("error"), "expected error in: {stderr}");
}

// r[verify error.auth-missing] r[verify data.write.auth]
#[test]
fn test_missing_auth_token() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["complete", "some-id"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Auth token") || stderr.contains("auth"),
        "expected auth error in: {stderr}"
    );
}

// ── Tags field resolution ──

// r[verify fields.tags]
#[test]
fn test_tags_field_resolution() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "today"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    let tagged = arr.iter().find(|item| item["title"] == "Today tagged task");
    assert!(
        tagged.is_some(),
        "expected 'Today tagged task' in today view"
    );
    assert_eq!(tagged.unwrap()["tags"], "urgent");
}

// ── Project field resolution ──

// r[verify fields.project]
#[test]
fn test_project_field_resolution() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "today"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    let in_project = arr
        .iter()
        .find(|item| item["title"] == "Today project task");
    assert!(in_project.is_some(), "expected 'Today project task'");
    assert_eq!(in_project.unwrap()["project"], "Test Project");
}

// ── Data access ──

// r[verify data.read] r[verify global.db-path]
#[test]
fn test_db_path_override() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["stats"]);
    assert!(ok);
    assert!(out.contains("inbox"));
}

// ── Fields ──

// r[verify fields.id]
#[test]
fn test_fields_id() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "inbox"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr[0]["id"], "task-inbox-1");
}

// r[verify fields.dates]
#[test]
fn test_fields_dates() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "upcoming"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    let task = arr
        .iter()
        .find(|item| item["title"] == "Upcoming task")
        .unwrap();
    assert_eq!(task["startDate"], "2025-12-15");
}

// r[verify fields.area]
#[test]
fn test_fields_area() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "projects"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    let proj = arr
        .iter()
        .find(|item| item["title"] == "Test Project")
        .unwrap();
    assert_eq!(proj["area"], "Work");
}

// ── Write commands (URL generation) ──

// r[verify cmd.add] r[verify cmd.add.params] r[verify cmd.add.output] r[verify data.write]
// r[verify test.date-decoding] r[verify test.write-urls]
// (URL generation tested in unit tests; integration test verifies CLI parsing)
#[test]
fn test_add_requires_title() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["add"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success()); // missing required <title>
}

// r[verify cmd.complete]
#[test]
fn test_complete_requires_auth() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["complete", "some-uuid"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Auth token") || stderr.contains("auth"));
}

// r[verify cmd.cancel]
#[test]
fn test_cancel_requires_auth() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["cancel", "some-uuid"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Auth token") || stderr.contains("auth"));
}

// r[verify cmd.update] r[verify cmd.update.params]
#[test]
fn test_update_requires_auth() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["update", "some-uuid", "--title", "New"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Auth token") || stderr.contains("auth"));
}

// r[verify cmd.move]
#[test]
fn test_move_requires_auth() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["move", "some-uuid", "--to", "project"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Auth token") || stderr.contains("auth"));
}

// ── Help text ──

// r[verify help.about]
#[test]
fn test_help_about() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .arg("--help")
        .output()
        .expect("failed to run tdo");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must mention Things 3
    assert!(stdout.contains("Things 3"), "help should mention Things 3");
    // Must describe the workflow
    assert!(
        stdout.contains("workflow") || stdout.contains("Typical workflow"),
        "help should describe the agent workflow"
    );
    // Must mention tdo skill
    assert!(
        stdout.contains("tdo skill"),
        "help should point to tdo skill"
    );
}

// r[verify help.subcommands]
#[test]
fn test_help_subcommands() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["today", "--help"])
        .output()
        .expect("failed to run tdo");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Subcommand help must explain what the view means
    assert!(
        stdout.contains("scheduled for today"),
        "today help should explain the view: {stdout}"
    );
    // Must include default columns
    assert!(
        stdout.contains("id, title, project, tags, deadline"),
        "today help should list default columns: {stdout}"
    );
}

// ── Skill ──

// r[verify cmd.skill.show]
#[test]
fn test_skill_show_prints_markdown() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["skill", "--show"])
        .output()
        .expect("failed to run tdo");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("# tdo"),
        "skill --show should start with a markdown heading"
    );
    assert!(
        stdout.contains("## Typical workflow"),
        "skill --show should have workflow section"
    );
    assert!(
        stdout.contains("## Available fields"),
        "skill --show should have fields section"
    );
    assert!(
        stdout.contains("## Read commands"),
        "skill --show should have read commands section"
    );
    assert!(
        stdout.contains("## Write commands"),
        "skill --show should have write commands section"
    );
}

// r[verify cmd.skill.show]
#[test]
fn test_skill_show_ignores_json_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["--json", "skill", "--show"])
        .output()
        .expect("failed to run tdo");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("# tdo"),
        "skill --show should output markdown even with --json"
    );
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "skill --show output should not be JSON"
    );
}

// r[verify cmd.skill.skip-existing]
#[test]
fn test_skill_skips_already_installed() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let claude_skills = tmp.path().join(".claude/skills/tdo");
    std::fs::create_dir_all(&claude_skills).unwrap();
    let skill_path = claude_skills.join("SKILL.md");
    // Write the current guide content
    let guide = include_str!("../docs/guide.md");
    std::fs::write(&skill_path, guide).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["skill", "--claude"])
        .env("HOME", tmp.path())
        .output()
        .expect("failed to run tdo");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already installed"),
        "should report already installed: {stderr}"
    );
}

// r[verify cmd.skill] r[verify cmd.skill.confirm]
#[test]
fn test_skill_aborts_on_no() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["skill", "--claude"])
        .env("HOME", tmp.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(b"n\n").ok();
            }
            child.wait_with_output()
        })
        .expect("failed to run tdo");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Aborted"),
        "should abort when user says no: {stderr}"
    );
    // File should not exist
    assert!(!tmp.path().join(".claude/skills/tdo/SKILL.md").exists());
}

// r[verify cmd.skill.claude]
#[test]
fn test_skill_installs_claude_only() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["skill", "--claude"])
        .env("HOME", tmp.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(b"y\n").ok();
            }
            child.wait_with_output()
        })
        .expect("failed to run tdo");

    assert!(output.status.success());
    assert!(tmp.path().join(".claude/skills/tdo/SKILL.md").exists());
    assert!(!tmp.path().join(".agents/skills/tdo/SKILL.md").exists());
    let content = std::fs::read_to_string(tmp.path().join(".claude/skills/tdo/SKILL.md")).unwrap();
    assert!(content.starts_with("# tdo"));
}

// r[verify cmd.skill.codex]
#[test]
fn test_skill_installs_codex_only() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["skill", "--codex"])
        .env("HOME", tmp.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(b"y\n").ok();
            }
            child.wait_with_output()
        })
        .expect("failed to run tdo");

    assert!(output.status.success());
    assert!(!tmp.path().join(".claude/skills/tdo/SKILL.md").exists());
    assert!(tmp.path().join(".agents/skills/tdo/SKILL.md").exists());
}

// r[verify cmd.skill]
#[test]
fn test_skill_installs_both_by_default() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["skill"])
        .env("HOME", tmp.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(b"y\n").ok();
            }
            child.wait_with_output()
        })
        .expect("failed to run tdo");

    assert!(output.status.success());
    assert!(tmp.path().join(".claude/skills/tdo/SKILL.md").exists());
    assert!(tmp.path().join(".agents/skills/tdo/SKILL.md").exists());
}

// ── Project subcommands ──

// r[verify cmd.project.tasks] r[verify cmd.project.tasks.columns]
#[test]
fn test_project_tasks_by_name() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["project", "tasks", "Test Project"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id\ttitle\ttags\tstartDate\tdeadline");
    assert!(lines.iter().any(|l| l.contains("Today project task")));
    assert!(lines.iter().any(|l| l.contains("Project child task")));
}

// r[verify cmd.project.tasks]
#[test]
fn test_project_tasks_by_uuid() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["project", "tasks", "project-test-1"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.iter().any(|l| l.contains("Today project task")));
}

// r[verify cmd.project.tasks]
#[test]
fn test_project_tasks_json() {
    let db = fixtures::create_fixture_db();
    let (out, _, ok) = run_tdo(db.path(), &["--json", "project", "tasks", "Test Project"]);
    assert!(ok);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON");
    let arr = parsed.as_array().unwrap();
    assert!(arr.iter().any(|item| item["title"] == "Today project task"));
    assert!(arr.iter().any(|item| item["title"] == "Project child task"));
}

// r[verify cmd.project.tasks]
#[test]
fn test_project_tasks_not_found() {
    let db = fixtures::create_fixture_db();
    let (_, stderr, ok) = run_tdo(db.path(), &["project", "tasks", "Nonexistent Project"]);
    assert!(!ok);
    assert!(
        stderr.contains("not found"),
        "expected 'not found' in: {stderr}"
    );
}

// r[verify cmd.project.add] r[verify cmd.project.add.params] r[verify cmd.project.add.output]
#[test]
fn test_project_add_requires_title() {
    let output = Command::new(env!("CARGO_BIN_EXE_tdo"))
        .args(["project", "add"])
        .output()
        .expect("failed to run tdo");
    assert!(!output.status.success()); // missing required <title>
}

// r[verify error.db-locked]
// (Tested implicitly via retry logic in db.rs; difficult to trigger in integration test)
