# tdo — Things 3 CLI Guide

A machine-friendly CLI for querying and writing to the Things 3 todo app on macOS.
Designed for AI agents, shell scripts, and automation.

## How it works

- **Read operations** query the Things 3 SQLite database directly (read-only).
- **Write operations** use the Things URL scheme (`things:///`) which briefly opens Things.
- Things 3 must be installed. The database is auto-discovered; override with `--db-path` or `TDO_DB_PATH`.

## Core concepts

Things 3 organizes tasks into views that reflect workflow stages:

| View         | Meaning                                           |
| ------------ | ------------------------------------------------- |
| **Inbox**    | Unprocessed tasks — not yet scheduled or assigned |
| **Today**    | Tasks scheduled for today                         |
| **Upcoming** | Tasks with a future start date                    |
| **Anytime**  | Started tasks available to work on (not in Today) |
| **Someday**  | Deferred tasks to revisit later                   |
| **Logbook**  | Completed tasks (most recent first)               |

Tasks can belong to **Projects** (groups of related tasks) and projects can belong to
**Areas** (high-level categories like "Work" or "Personal"). Tasks can also have
**Tags**, **Deadlines**, **Checklist items**, and **Notes**.

## Limitations

- **Projects** can be created via `tdo project add` but cannot be deleted or archived via CLI — use the Things UI for that.
- **Project properties** (title, notes, etc.) cannot be modified after creation via CLI.
- **Tags and areas** can only be listed (`tdo tags`, `tdo areas`), not created or modified via CLI.
- **Write operations** use the Things URL scheme, which briefly opens Things. There may be a short delay before the database reflects the change.
- `--list` and `--to` accept a project name (recommended) or project UUID.

## Typical workflow

1. **List tasks** to see items and their UUIDs:

   ```
   tdo today
   tdo inbox
   tdo projects
   ```

2. **Inspect** a specific item by UUID:

   ```
   tdo show <uuid>
   ```

3. **Modify** items:
   ```
   tdo complete <uuid>
   tdo update <uuid> --title "New title"
   tdo add "Buy groceries" --when today --tags "errands"
   ```

## Output formats

### TSV (default)

Tab-separated values with a header row. Each subcommand has sensible default columns.

```
$ tdo today
id	title	project	tags	deadline
ABC-123	Fix login bug	Backend	urgent	2025-06-15
DEF-456	Write tests	Backend
```

### JSON (`--json`)

Array of objects with all available fields (not just default columns).

```
$ tdo today --json
[{"id":"ABC-123","title":"Fix login bug","project":"Backend","tags":"urgent","deadline":"2025-06-15",...}]
```

### Custom columns (`--fields`)

Override default columns with a comma-separated list:

```
tdo today --fields id,title,deadline
```

### Suppress header (`--no-header`)

Omit the TSV header row (useful for piping):

```
tdo today --no-header | cut -f2
```

## Read commands

### tdo inbox

List unprocessed tasks not yet assigned to a project or scheduled.
Default columns: `id`, `title`, `tags`, `deadline`

### tdo today

List tasks scheduled for today.
Default columns: `id`, `title`, `project`, `tags`, `deadline`

### tdo upcoming

List tasks with a future start date, ordered by date.
Default columns: `id`, `title`, `project`, `tags`, `startDate`, `deadline`

### tdo anytime

List started tasks not in Today — available to work on.
Default columns: `id`, `title`, `project`, `area`, `tags`, `deadline`

### tdo someday

List deferred tasks to revisit later.
Default columns: `id`, `title`, `project`, `tags`

### tdo logbook [--limit N]

List completed tasks, most recent first. Default limit: 50.
Default columns: `id`, `title`, `project`, `completedDate`

### tdo projects

List all open projects.
Default columns: `id`, `title`, `area`, `tags`, `deadline`, `openTasks`

### tdo areas

List all areas.
Default columns: `id`, `title`

### tdo tags

List all tags.
Default columns: `id`, `title`, `shortcut`, `parent`

### tdo project tasks \<project\>

List all open tasks belonging to a project. Accepts project name (case-insensitive) or UUID.
Default columns: `id`, `title`, `tags`, `startDate`, `deadline`

```
tdo project tasks "My Project"
tdo project tasks project-uuid-here
```

### tdo show \<id\>

Show full details of a single item by UUID. Includes notes, checklist items, and tags.
In TSV mode: key-value pairs (one per line). In JSON mode: single object with nested arrays.

### tdo search \<query\>

Search tasks and projects by title or notes (case-insensitive substring match).
Default columns: `id`, `title`, `project`, `status`, `tags`

### tdo stats

Show database statistics: counts of items by status, projects, areas, tags.

## Write commands

Write commands use the Things URL scheme and briefly open the Things app.
Commands that modify existing items require an auth token.

### Auth token setup

Set once via environment variable (recommended):

```
export TDO_AUTH_TOKEN="your-token-here"
```

Or per-command:

```
tdo complete <id> --auth-token "your-token"
```

Find your token in: **Things > Settings > General > Enable Things URLs > Authentication Token**

### tdo project add \<title\>

Create a new project. Does not require an auth token.

```
tdo project add "Sprint 13"
tdo project add "House Renovation" --area "Personal" --deadline 2025-09-01
tdo project add "Launch Plan" --todo "Write copy" --todo "Design assets" --todo "QA pass"
```

Flags: `--notes`, `--when` (today/tomorrow/evening/anytime/someday/YYYY-MM-DD),
`--deadline` (YYYY-MM-DD), `--tags` (comma-separated), `--area` (area name or ID),
`--todo` (repeatable, adds tasks to the project).

### tdo add \<title\>

Create a new task. Does not require an auth token.

```
tdo add "Buy groceries"
tdo add "Review PR" --when today --tags "work" --list "Sprint 12"
tdo add "Pack for trip" --checklist-item "Passport" --checklist-item "Charger"
tdo add "Submit report" --when 2025-07-01 --deadline 2025-07-15
```

Flags: `--notes`, `--when` (today/tomorrow/evening/anytime/someday/YYYY-MM-DD),
`--deadline` (YYYY-MM-DD), `--tags` (comma-separated), `--list` (project name or ID),
`--heading`, `--checklist-item` (repeatable).

### tdo complete \<id\>

Mark a task as completed. Requires auth token.

### tdo cancel \<id\>

Mark a task as cancelled. Requires auth token.

### tdo update \<id\>

Update an existing task's properties. Requires auth token.

```
tdo update <id> --title "New title"
tdo update <id> --when today
tdo update <id> --append-notes "Additional context here"
tdo update <id> --add-tags "priority,review"
```

Flags: `--title`, `--notes` (replace), `--append-notes`, `--prepend-notes`,
`--when`, `--deadline`, `--add-tags` (comma-separated), `--list`, `--heading`.

### tdo move \<id\> --to \<list\>

Move a task to a different project or area. Requires auth token.

## Available fields

These field names can be used with `--fields`:

| Field           | Description                                    |
| --------------- | ---------------------------------------------- |
| `id`            | UUID — the primary identifier for all commands |
| `title`         | Task or project title                          |
| `project`       | Parent project title                           |
| `area`          | Parent area title                              |
| `tags`          | Comma-separated tag names                      |
| `startDate`     | Scheduled start date (YYYY-MM-DD)              |
| `deadline`      | Deadline date (YYYY-MM-DD)                     |
| `completedDate` | Completion date (YYYY-MM-DD)                   |
| `status`        | Task status (open/completed/cancelled)         |
| `openTasks`     | Number of open child tasks (projects only)     |

## Tips for AI agents

- **Get UUIDs first**: All item-specific commands need a UUID. Get them from list commands.
- **Use `--json` for parsing**: JSON output includes all fields and is easier to parse programmatically.
- **Search before acting**: Use `tdo search` to find items by keyword if you don't have a UUID.
- **Check projects**: Use `tdo projects` to see available projects before assigning tasks with `--list`.
- **Batch reading**: Run multiple list commands to build a complete picture of the user's task state.
- **Auth token**: Set `TDO_AUTH_TOKEN` once in your environment to avoid passing it on every write command.

## Common workflows

### Create a project with tasks

```
tdo project add "Sprint 13" --todo "Design API" --todo "Implement endpoints" --todo "Write tests"
```

### List tasks in a project

```
tdo project tasks "Sprint 13"
tdo project tasks "Sprint 13" --json
```

### Move tasks between projects

```
# Get task IDs from one project, then move them
tdo project tasks "Old Project" --no-header --fields id | while read id; tdo move "$id" --to "New Project"; end
```
