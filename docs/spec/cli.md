# tdo — Things 3 CLI

A machine-friendly command-line interface for interacting with the Things 3 todo app on macOS. Designed primarily as a tool for AI agents and shell scripting, with human-readable output available when needed.

## Help text

r[help.about]
The top-level `--help` must include a brief description of Things 3 and the typical agent workflow:
list tasks to get UUIDs, then use UUIDs with show/complete/update/cancel.
Run `tdo guide` for a comprehensive reference.

r[help.subcommands]
Each subcommand's help text must describe what the command does in enough detail for an
AI agent unfamiliar with Things 3 to understand it, including the default output columns.

## Data access

r[data.read]
All read operations query the Things 3 SQLite database directly at
`~/Library/Group Containers/JLMPQHK86H.com.culturedcode.ThingsMac/ThingsData-*/Things Database.thingsdatabase/main.sqlite`.
The database MUST be opened in read-only mode.

r[data.write]
All write operations MUST use the Things URL scheme (`things:///`), never direct SQLite writes.
This ensures data integrity and compatibility with Things Cloud sync.

r[data.write.auth]
Write operations that modify existing items (update, complete, cancel) require an auth token.
The auth token MUST be configurable via the `--auth-token` flag or the `TDO_AUTH_TOKEN` environment variable.

## Output format

r[output.tsv]
The default output format is TSV (tab-separated values) with a header row.
Each subcommand defines sensible default columns.

r[output.tsv.fields]
The `--fields` flag overrides the default columns for any subcommand.
Fields are specified as a comma-separated list of column names (e.g. `--fields id,title,project,deadline`).

r[output.json]
The `--json` flag switches output to JSON (an array of objects).
JSON output includes all available fields for each item, not just the default columns.

r[output.no-header]
The `--no-header` flag suppresses the TSV header row. Has no effect in JSON mode.

## Global flags

r[global.db-path]
The `--db-path` flag overrides the default database path.
This is useful for testing or when the database is in a non-standard location.

## Read subcommands

### tdo inbox

r[cmd.inbox]
Lists all items in the Inbox (tasks with `start=0`, `status=0`, `trashed=0`, no project, no area, `type=0`).

r[cmd.inbox.columns]
Default columns: `id`, `title`, `tags`, `deadline`.

### tdo today

r[cmd.today]
Lists all items in the Today view. This matches Things' own Today view exactly:
tasks with `start=1`, `status=0`, `trashed=0`, `type=0`.

r[cmd.today.columns]
Default columns: `id`, `title`, `project`, `tags`, `deadline`.

### tdo upcoming

r[cmd.upcoming]
Lists items with a future `startDate`, `status=0`, `trashed=0`, `type=0`, ordered by `startDate` ascending.

r[cmd.upcoming.columns]
Default columns: `id`, `title`, `project`, `tags`, `startDate`, `deadline`.

### tdo anytime

r[cmd.anytime]
Lists items with `start=1`, `status=0`, `trashed=0`, `type=0`, that are NOT in Today
(i.e. have no `todayIndex` set for the current date). Includes items in projects and areas.

r[cmd.anytime.columns]
Default columns: `id`, `title`, `project`, `area`, `tags`, `deadline`.

### tdo someday

r[cmd.someday]
Lists items with `start=2`, `status=0`, `trashed=0`, `type=0`.

r[cmd.someday.columns]
Default columns: `id`, `title`, `project`, `tags`.

### tdo logbook

r[cmd.logbook]
Lists completed items (`status=3`, `trashed=0`), ordered by `stopDate` descending.

r[cmd.logbook.limit]
Defaults to showing the 50 most recent completed items. The `--limit` flag overrides this.

r[cmd.logbook.columns]
Default columns: `id`, `title`, `project`, `completedDate`.

### tdo projects

r[cmd.projects]
Lists all open projects (`type=1`, `status=0`, `trashed=0`).

r[cmd.projects.columns]
Default columns: `id`, `title`, `area`, `tags`, `deadline`, `openTasks` (count of open child tasks).

### tdo areas

r[cmd.areas]
Lists all visible areas from the `TMArea` table.

r[cmd.areas.columns]
Default columns: `id`, `title`.

### tdo tags

r[cmd.tags]
Lists all tags from the `TMTag` table.

r[cmd.tags.columns]
Default columns: `id`, `title`, `shortcut`, `parent`.

### tdo show \<id\>

r[cmd.show]
Shows the full details of a single item (task or project) by its UUID.
Includes all fields, notes, checklist items, and tags.

r[cmd.show.output]
In TSV mode, outputs key-value pairs (one per line, key\tvalue).
In JSON mode, outputs a single object with all fields including nested `checklistItems` and `tags` arrays.

### tdo search \<query\>

r[cmd.search]
Full-text search across task and project titles and notes.
The query is matched case-insensitively using SQL LIKE.

r[cmd.search.columns]
Default columns: `id`, `title`, `project`, `status`, `tags`.

### tdo stats

r[cmd.stats]
Prints a summary of the Things database: counts of items by status, projects, areas, tags.

r[cmd.stats.output]
In TSV mode, outputs key-value pairs. In JSON mode, a single summary object.

### tdo guide

r[cmd.guide]
Prints a detailed markdown guide explaining Things 3 concepts, the tdo workflow,
all subcommands with examples, available fields, output format details, and
common patterns. Designed to be consumed by AI agents as a reference document.

r[cmd.guide.output]
Output is always plain markdown text (not TSV/JSON). The `--json` and `--fields` flags have no effect.

## Write subcommands

### tdo add \<title\>

r[cmd.add]
Creates a new todo via the Things URL scheme (`things:///add`).

r[cmd.add.params]
Supports flags: `--notes`, `--when` (today/tomorrow/evening/anytime/someday/date),
`--deadline`, `--tags` (comma-separated), `--list` (project name or id),
`--heading`, `--checklist-items` (newline-separated or repeated flag).

r[cmd.add.output]
On success, prints the ID of the created item (returned via x-callback-url).

### tdo complete \<id\>

r[cmd.complete]
Marks an item as completed via `things:///update` with `completed=true`.
Requires auth token.

### tdo cancel \<id\>

r[cmd.cancel]
Marks an item as canceled via `things:///update` with `canceled=true`.
Requires auth token.

### tdo update \<id\>

r[cmd.update]
Updates an existing item via `things:///update`.
Requires auth token.

r[cmd.update.params]
Supports flags: `--title`, `--notes`, `--append-notes`, `--prepend-notes`,
`--when`, `--deadline`, `--add-tags`, `--list`, `--heading`.

### tdo project add \<title\>

r[cmd.project.add]
Creates a new project via the Things URL scheme (`things:///add-project`).
Does not require an auth token.

r[cmd.project.add.params]
Supports flags: `--notes`, `--when` (today/tomorrow/evening/anytime/someday/date),
`--deadline`, `--tags` (comma-separated), `--area` (area name or id),
`--todo` (repeatable, adds tasks to the project).

r[cmd.project.add.output]
On success, prints the title of the created project.

### tdo project tasks \<project\>

r[cmd.project.tasks]
Lists all open tasks belonging to a given project. The project can be specified
by title (case-insensitive match) or UUID. Resolves project by name first (query
TMTask where type=1 and title matches), then uses the UUID to find child tasks.

r[cmd.project.tasks.columns]
Default columns: `id`, `title`, `tags`, `startDate`, `deadline`.

### tdo move \<id\> --to \<list\>

r[cmd.move]
Moves an item to a different project or area via `things:///update` with `list` parameter.
Requires auth token.

## Field reference

r[fields.id]
The `id` field is the item's UUID from the database. This is the primary identifier used
in `show`, `complete`, `update`, and other item-specific commands.

r[fields.dates]
Date fields (`startDate`, `deadline`, `completedDate`) are output as `YYYY-MM-DD` strings.
The internal Things date format (packed integers) MUST be decoded correctly.

r[fields.tags]
The `tags` field is a comma-separated list of tag titles. Resolved by joining
`TMTaskTag` and `TMTag` tables.

r[fields.project]
The `project` field shows the parent project's title (resolved from the `project` UUID
in `TMTask` via a self-join on `TMTask` where `type=1`).

r[fields.area]
The `area` field shows the parent area's title (resolved from the `area` UUID
in `TMTask` via a join on `TMArea`).

## Testing

r[test.fixture-db]
Tests use an in-memory or temporary SQLite database populated with known fixture data
that mirrors the Things 3 schema. Tests MUST NOT depend on a real Things installation.

r[test.fixtures]
A test fixture module provides helper functions to create a database with the full Things schema
and populate it with sample areas, projects, tags, tasks, and checklist items covering
all edge cases (inbox items, today items, someday items, completed items, items with deadlines, etc.).

r[test.read-commands]
Every read subcommand (inbox, today, upcoming, anytime, someday, logbook, projects, areas,
tags, show, search, stats) MUST have integration tests that run against fixture data
and assert on both TSV and JSON output.

r[test.output-formats]
Tests MUST verify that TSV output has correct headers, tab-separated fields, and that
`--no-header` suppresses the header. Tests MUST verify that `--json` produces valid JSON
with all expected fields. Tests MUST verify that `--fields` correctly limits columns.

r[test.date-decoding]
The date decoding logic MUST have unit tests covering known date values
to ensure the packed integer format is correctly converted to `YYYY-MM-DD`.

r[test.write-urls]
Write command tests MUST verify that the correct Things URL scheme URLs are generated
without actually opening them. The URL-opening mechanism MUST be injectable/mockable
so tests can capture the URL that would be opened.

r[test.error-cases]
Tests MUST cover error paths: missing database, missing auth token for write commands,
and invalid item IDs for show/complete/update.

## Error handling

r[error.db-not-found]
If the Things database cannot be found at the expected path, print a clear error message
to stderr and exit with code 1. Suggest using `--db-path` to specify the location manually.

r[error.db-locked]
If the database is locked (Things is performing a sync), retry up to 3 times with a
100ms delay, then fail with a clear error message.

r[error.auth-missing]
If a write command is invoked without an auth token, print an error message explaining
how to set one (via flag or environment variable) and exit with code 1.
