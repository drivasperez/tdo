# tdo — Things 3 CLI

## Project overview

A machine-friendly CLI for querying and writing to the Things 3 SQLite database on macOS. Primarily for AI agent consumption.

## Key references

- Spec: `docs/spec/cli.md` — 56 tracey requirements
- Schema: `docs/things3-schema.md` — SQLite schema and date encoding details

## VCS

Use `jj` (Jujutsu) for version control, not `git`. Use `jj add` instead of `git add`, etc.

## Build & test

```
cargo build
cargo test
```

## Workflow rule: spec before code

Before implementing any new feature or changing behavior:
1. Check that a tracey requirement exists in `docs/spec/cli.md` covering the change
2. If no requirement exists, add one first and get approval
3. Only then implement the code with the matching `r[requirement.name]` annotation
4. Add `r[verify requirement.name]` in the corresponding test

Never implement functionality that isn't covered by a spec requirement.

## Architecture

```
src/
  main.rs     — clap CLI definition, subcommand dispatch
  db.rs       — database connection, path discovery, retry logic
  model.rs    — Task, Project, Area, Tag, ChecklistItem structs
  queries.rs  — SQL queries for each view (inbox, today, etc.)
  output.rs   — TSV and JSON formatters, --fields handling
  dates.rs    — Things date integer decoding (bit-packed → YYYY-MM-DD)
  write.rs    — Things URL scheme command builders
  error.rs    — Error types and display
```

## Tracey (requirements traceability)

Config: `.config/tracey/config.styx`

### Commands

```
tracey query status      # coverage + verification summary
tracey query uncovered   # list requirements with no impl annotation
tracey query untested    # list requirements with no verify annotation
tracey query validate    # check for annotation errors
```

### Annotation format

- **In spec files** (`docs/spec/**/*.md`): `r[requirement.name]` defines a requirement
- **In source files** (`src/**/*.rs`): `r[requirement.name]` marks implementation coverage
- **In test files** (`tests/**/*.rs`): `r[verify requirement.name]` marks verification

Multiple annotations can appear on a single line: `// r[foo] r[bar]`

### Known bug (tracey 1.1.5)

Files listed only in `test_include` are discovered but not parsed for refs. As a workaround, test files must appear in **both** `include` and `test_include`. See `docs/tracey-bug.md` for details.
