# tdo

A machine-friendly CLI for querying and writing to the [Things 3](https://culturedcode.com/things/) database on macOS. Designed for AI agents and shell scripting, with human-readable output available too.

## Install

```
cargo install --path .
```

Or build from source:

```
cargo build --release
```

## Quick start

Things 3 organizes tasks into views: Inbox, Today, Upcoming, Anytime, Someday, and Logbook. Tasks belong to Projects, which belong to Areas. Tasks can have tags, deadlines, and checklist items.

Typical workflow:

1. List tasks to see items and their UUIDs: `tdo today`, `tdo inbox`
2. Inspect a specific item: `tdo show <uuid>`
3. Modify items: `tdo complete <uuid>`, `tdo update <uuid>`, etc.

## Output formats

Output is **TSV** by default with a header row. Each subcommand has sensible default columns.

```
$ tdo today
id	title	project	tags	deadline
ABC-123	Write README	tdo	dev	2026-03-05
```

Use `--json` for JSON output (includes all fields):

```
$ tdo today --json
```

Use `--fields` to pick specific columns:

```
$ tdo today --fields id,title,deadline
```

Use `--no-header` to suppress the TSV header row.

## Read commands

| Command | Description | Default columns |
|---------|-------------|-----------------|
| `tdo inbox` | Unprocessed tasks | id, title, tags, deadline |
| `tdo today` | Tasks scheduled for today | id, title, project, tags, deadline |
| `tdo upcoming` | Tasks with a future start date | id, title, project, tags, startDate, deadline |
| `tdo anytime` | Started tasks not in Today | id, title, project, area, tags, deadline |
| `tdo someday` | Deferred tasks | id, title, project, tags |
| `tdo logbook` | Completed items (most recent first) | id, title, project, completedDate |
| `tdo projects` | Open projects | id, title, area, tags, deadline, openTasks |
| `tdo areas` | Areas of responsibility | id, title |
| `tdo tags` | All tags | id, title, shortcut, parent |
| `tdo show <id>` | Full details of a single item | All fields (key-value in TSV, object in JSON) |
| `tdo search <query>` | Search titles and notes | id, title, project, status, tags |
| `tdo stats` | Database summary counts | Key-value pairs |
| `tdo project tasks <project>` | Tasks in a project (by name or UUID) | id, title, tags, startDate, deadline |

The logbook defaults to 50 items; override with `--limit`.

## Write commands

Write commands use the [Things URL scheme](https://culturedcode.com/things/support/articles/2803573/) and briefly open Things to apply changes.

| Command | Description | Auth required |
|---------|-------------|---------------|
| `tdo add <title>` | Create a new todo | No |
| `tdo project add <title>` | Create a new project | No |
| `tdo complete <id>` | Mark item as completed | Yes |
| `tdo cancel <id>` | Mark item as cancelled | Yes |
| `tdo update <id>` | Update an existing item | Yes |
| `tdo move <id> --to <list>` | Move item to a project/area | Yes |

### Auth token

Commands that modify existing items require an auth token. Set it via:

- Flag: `--auth-token <TOKEN>`
- Environment variable: `TDO_AUTH_TOKEN`

You can find the auth token in Things > Settings > General > Enable Things URLs.

### Adding a todo

```
tdo add "Buy groceries" --when today --tags errands --notes "Milk, eggs, bread"
tdo add "Review PR" --list "tdo" --deadline 2026-03-10 --checklist-item "Check tests" --checklist-item "Review docs"
```

### Updating a todo

```
tdo update <uuid> --title "New title" --append-notes "Additional context" --add-tags urgent
```

## Global flags

| Flag | Description |
|------|-------------|
| `--db-path <PATH>` | Override the default database path (also `TDO_DB_PATH` env var) |
| `--json` | Output as JSON |
| `--fields <FIELDS>` | Override default columns (comma-separated) |
| `--no-header` | Suppress TSV header row |
| `--auth-token <TOKEN>` | Auth token for write operations (also `TDO_AUTH_TOKEN` env var) |

## AI agent skill

tdo includes a built-in guide for AI agents:

```
tdo skill --show    # Print the guide to stdout
tdo skill           # Install as a skill file for Claude Code and Codex
tdo skill --claude  # Install for Claude Code only
tdo skill --codex   # Install for Codex only
```

## How it works

- **Reads** query the Things 3 SQLite database directly (read-only mode)
- **Writes** use the Things URL scheme (`things:///`) to ensure data integrity and sync compatibility
- The database is auto-discovered at `~/Library/Group Containers/JLMPQHK86H.com.culturedcode.ThingsMac/`

## Development

```
cargo build
cargo test
```

## License

See [LICENSE](LICENSE) for details.
