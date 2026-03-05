mod dates;
mod db;
mod error;
mod model;
mod output;
mod queries;
mod write;

use clap::{Parser, Subcommand};
use error::Error;
use output::{Header, TsvConfig};
use write::{AddParams, AddProjectParams, MacOsUrlOpener, UpdateParams, UrlOpener};

// r[help.about]
#[derive(Parser)]
#[command(
    name = "tdo",
    about = "Things 3 CLI — machine-friendly interface for querying and writing to the Things 3 todo app on macOS",
    long_about = "Things 3 CLI — machine-friendly interface for querying and writing to the Things 3 todo app on macOS.\n\n\
        Things 3 organizes tasks into views: Inbox, Today, Upcoming, Anytime, Someday, and Logbook.\n\
        Tasks belong to Projects, which belong to Areas. Tasks can have tags, deadlines, and checklist items.\n\n\
        Typical workflow:\n  \
          1. List tasks (e.g. `tdo today`, `tdo inbox`) to see items and their UUIDs\n  \
          2. Inspect a specific item with `tdo show <uuid>`\n  \
          3. Modify items with `tdo complete <uuid>`, `tdo update <uuid>`, etc.\n\n\
        Output is TSV by default (use --json for JSON). Use `tdo skill --show` for a comprehensive reference, or `tdo skill` to install the guide as an AI agent skill."
)]
struct Cli {
    /// Override the default database path
    // r[global.db-path]
    #[arg(long, global = true, env = "TDO_DB_PATH")]
    db_path: Option<String>,

    /// Output as JSON instead of TSV
    // r[output.json]
    #[arg(long, global = true)]
    json: bool,

    /// Override default output columns (comma-separated)
    // r[output.tsv.fields]
    #[arg(long, global = true)]
    fields: Option<String>,

    /// Suppress TSV header row
    // r[output.no-header]
    #[arg(long, global = true)]
    no_header: bool,

    /// Auth token for write operations
    // r[data.write.auth]
    #[arg(long, global = true, env = "TDO_AUTH_TOKEN")]
    auth_token: Option<String>,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    fn header(&self) -> Header {
        if self.no_header {
            Header::Hide
        } else {
            Header::Show
        }
    }

    fn tsv_config<'a>(&'a self, default_fields: &'a [&'a str]) -> TsvConfig<'a> {
        TsvConfig {
            default_fields,
            fields: &self.fields,
            header: self.header(),
        }
    }
}

// r[help.subcommands]
#[derive(Subcommand)]
enum Command {
    /// List inbox items (unprocessed tasks not yet assigned to a project or scheduled).
    /// Default columns: id, title, tags, deadline
    Inbox,
    /// List today items (tasks scheduled for today).
    /// Default columns: id, title, project, tags, deadline
    Today,
    /// List upcoming items (tasks with a future start date, ordered by date).
    /// Default columns: id, title, project, tags, startDate, deadline
    Upcoming,
    /// List anytime items (started tasks not in Today — available to work on).
    /// Default columns: id, title, project, area, tags, deadline
    Anytime,
    /// List someday items (deferred tasks to revisit later).
    /// Default columns: id, title, project, tags
    Someday,
    /// List completed items from the logbook, most recent first.
    /// Default columns: id, title, project, completedDate
    Logbook {
        /// Maximum number of items to show (default: 50)
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// List open projects (containers that group related tasks).
    /// Default columns: id, title, area, tags, deadline, openTasks
    Projects,
    /// List areas (high-level categories like "Work" or "Personal" that group projects).
    /// Default columns: id, title
    Areas,
    /// List all tags.
    /// Default columns: id, title, shortcut, parent
    Tags,
    /// Show full details of a single item by UUID (includes notes, checklist, tags).
    /// In TSV mode: key-value pairs. In JSON mode: single object with nested arrays
    Show {
        /// Item UUID (get UUIDs from list commands like `tdo today`)
        id: String,
    },
    /// Search tasks and projects by title or notes (case-insensitive substring match).
    /// Default columns: id, title, project, status, tags
    Search {
        /// Search query (matched case-insensitively against title and notes)
        query: String,
    },
    /// Show database statistics (counts of items by status, projects, areas, tags)
    Stats,
    // r[cmd.skill] r[cmd.skill.claude] r[cmd.skill.codex] r[cmd.skill.show]
    // r[cmd.skill.confirm] r[cmd.skill.skip-existing]
    /// Install the tdo guide as an AI agent skill, or print it with --show
    Skill {
        /// Only install to ~/.claude/skills/ (Claude Code)
        #[arg(long)]
        claude: bool,
        /// Only install to ~/.agents/skills/ (Codex)
        #[arg(long)]
        codex: bool,
        /// Print the guide to stdout instead of installing
        #[arg(long)]
        show: bool,
    },
    /// Add a new todo via the Things URL scheme (opens Things briefly).
    Add {
        /// Todo title
        title: String,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
        /// When (today/tomorrow/evening/anytime/someday/date)
        #[arg(long)]
        when: Option<String>,
        /// Deadline date
        #[arg(long)]
        deadline: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Project name or ID
        #[arg(long)]
        list: Option<String>,
        /// Heading within project
        #[arg(long)]
        heading: Option<String>,
        /// Checklist items (can be repeated)
        #[arg(long = "checklist-item")]
        checklist_items: Vec<String>,
    },
    /// Mark an item as completed (requires --auth-token or TDO_AUTH_TOKEN)
    Complete {
        /// Item UUID (get UUIDs from list commands like `tdo today`)
        id: String,
    },
    /// Mark an item as cancelled (requires --auth-token or TDO_AUTH_TOKEN)
    Cancel {
        /// Item UUID (get UUIDs from list commands like `tdo today`)
        id: String,
    },
    /// Update an existing item's title, notes, dates, or tags (requires auth token)
    Update {
        /// Item UUID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// Replace notes
        #[arg(long)]
        notes: Option<String>,
        /// Append to notes
        #[arg(long)]
        append_notes: Option<String>,
        /// Prepend to notes
        #[arg(long)]
        prepend_notes: Option<String>,
        /// When (today/tomorrow/evening/anytime/someday/date)
        #[arg(long)]
        when: Option<String>,
        /// Deadline date
        #[arg(long)]
        deadline: Option<String>,
        /// Add tags (comma-separated)
        #[arg(long)]
        add_tags: Option<String>,
        /// Move to project/area
        #[arg(long)]
        list: Option<String>,
        /// Heading within project
        #[arg(long)]
        heading: Option<String>,
    },
    /// Move an item to a different project/area
    Move {
        /// Item UUID
        id: String,
        /// Target project or area
        #[arg(long)]
        to: String,
    },
    /// Manage projects (create, list tasks)
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
}

#[derive(Subcommand)]
enum ProjectCommand {
    /// Create a new project via the Things URL scheme (opens Things briefly).
    /// Does not require an auth token.
    Add {
        /// Project title
        title: String,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
        /// When (today/tomorrow/evening/anytime/someday/date)
        #[arg(long)]
        when: Option<String>,
        /// Deadline date
        #[arg(long)]
        deadline: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Area name or ID
        #[arg(long)]
        area: Option<String>,
        /// Add a task to the project (can be repeated)
        #[arg(long = "todo")]
        todos: Vec<String>,
    },
    /// List all open tasks in a project (by name or UUID).
    /// Default columns: id, title, tags, startDate, deadline
    Tasks {
        /// Project name or UUID
        project: String,
    },
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        // Skill command — no DB or URL scheme needed
        Command::Skill {
            claude,
            codex,
            show,
        } => run_skill(*claude, *codex, *show),
        // Read commands need DB access
        Command::Inbox
        | Command::Today
        | Command::Upcoming
        | Command::Anytime
        | Command::Someday
        | Command::Logbook { .. }
        | Command::Projects
        | Command::Areas
        | Command::Tags
        | Command::Show { .. }
        | Command::Search { .. }
        | Command::Stats => {
            let db_path = match &cli.db_path {
                Some(p) => p.clone(),
                None => db::find_db_path()?,
            };
            let conn = db::open_db(&db_path)?;
            run_read_command(&cli, &conn)
        }
        // Write commands use URL scheme
        Command::Add { .. }
        | Command::Complete { .. }
        | Command::Cancel { .. }
        | Command::Update { .. }
        | Command::Move { .. } => run_write_command(&cli, &MacOsUrlOpener),
        // Project subcommands — mixed read/write
        Command::Project { command } => match command {
            ProjectCommand::Tasks { .. } => {
                let db_path = match &cli.db_path {
                    Some(p) => p.clone(),
                    None => db::find_db_path()?,
                };
                let conn = db::open_db(&db_path)?;
                run_read_command(&cli, &conn)
            }
            ProjectCommand::Add { .. } => run_write_command(&cli, &MacOsUrlOpener),
        },
    }
}

const GUIDE_CONTENT: &str = include_str!("../docs/guide.md");
const SKILL_DIR: &str = "tdo";
const SKILL_FILENAME: &str = "SKILL.md";

enum SkillTarget {
    Claude,
    Codex,
    Both,
}

fn skill_paths(target: &SkillTarget) -> Vec<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
    let claude = std::path::PathBuf::from(&home)
        .join(".claude/skills")
        .join(SKILL_DIR)
        .join(SKILL_FILENAME);
    let codex = std::path::PathBuf::from(&home)
        .join(".agents/skills")
        .join(SKILL_DIR)
        .join(SKILL_FILENAME);
    match target {
        SkillTarget::Claude => vec![claude],
        SkillTarget::Codex => vec![codex],
        SkillTarget::Both => vec![claude, codex],
    }
}

// ANSI escape helpers
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

fn run_skill(claude: bool, codex: bool, show: bool) -> Result<(), Error> {
    if show {
        print!("{}", GUIDE_CONTENT);
        return Ok(());
    }

    let target = match (claude, codex) {
        (true, false) => SkillTarget::Claude,
        (false, true) => SkillTarget::Codex,
        _ => SkillTarget::Both,
    };

    let paths = skill_paths(&target);

    // Check which paths actually need installing
    let mut to_install: Vec<&std::path::PathBuf> = Vec::new();
    for path in &paths {
        if path.exists() && std::fs::read_to_string(path).unwrap_or_default() == GUIDE_CONTENT {
            eprintln!(
                "  {DIM}\u{2500}\u{2500}{RESET} {CYAN}{}{RESET} {DIM}\u{2014} already installed{RESET}",
                path.display()
            );
        } else {
            to_install.push(path);
        }
    }

    if to_install.is_empty() {
        eprintln!("\n  {GREEN}\u{2713}{RESET} Nothing to do.");
        return Ok(());
    }

    // Prompt for confirmation
    eprintln!("\n  {BOLD}tdo{RESET} {DIM}\u{00b7}{RESET} skill installer\n");
    for path in &to_install {
        eprintln!("  {YELLOW}\u{25b8}{RESET} {CYAN}{}{RESET}", path.display());
    }
    eprintln!();
    eprint!("  {BOLD}Install?{RESET} {DIM}[y/N]{RESET} ");
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if !answer.trim().eq_ignore_ascii_case("y") {
        eprintln!("\n  {RED}\u{00d7}{RESET} Aborted.");
        return Ok(());
    }

    eprintln!();
    for path in &to_install {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, GUIDE_CONTENT)?;
        eprintln!(
            "  {GREEN}\u{2500}\u{2500}{RESET} {CYAN}{}{RESET} {GREEN}\u{2713}{RESET}",
            path.display()
        );
    }
    eprintln!("\n  {GREEN}{BOLD}\u{2234}{RESET} Done.\n");

    Ok(())
}

fn run_read_command(cli: &Cli, conn: &rusqlite::Connection) -> Result<(), Error> {
    match &cli.command {
        Command::Inbox => {
            let rows = queries::inbox(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "tags", "deadline"]))?;
            }
        }
        Command::Today => {
            let rows = queries::today(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "project", "tags", "deadline"]),
                )?;
            }
        }
        Command::Upcoming => {
            let rows = queries::upcoming(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "project", "tags", "startDate", "deadline"]),
                )?;
            }
        }
        Command::Anytime => {
            let rows = queries::anytime(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "project", "area", "tags", "deadline"]),
                )?;
            }
        }
        Command::Someday => {
            let rows = queries::someday(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "project", "tags"]))?;
            }
        }
        Command::Logbook { limit } => {
            let rows = queries::logbook(conn, *limit)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "project", "completedDate"]),
                )?;
            }
        }
        Command::Projects => {
            let rows = queries::projects(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "area", "tags", "deadline", "openTasks"]),
                )?;
            }
        }
        Command::Areas => {
            let rows = queries::areas(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title"]))?;
            }
        }
        Command::Tags => {
            let rows = queries::tags(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "shortcut", "parent"]),
                )?;
            }
        }
        Command::Show { id } => {
            let row = queries::show(conn, id)?;
            if cli.json {
                output::print_show_json(&row)?;
            } else {
                output::print_show_tsv(&row, cli.header())?;
            }
        }
        Command::Search { query } => {
            let rows = queries::search(conn, query)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "project", "status", "tags"]),
                )?;
            }
        }
        Command::Stats => {
            let kvs = queries::stats(conn)?;
            if cli.json {
                output::print_kv_json(&kvs)?;
            } else {
                output::print_kv_tsv(&kvs, cli.header())?;
            }
        }
        Command::Project {
            command: ProjectCommand::Tasks { project },
        } => {
            let rows = queries::project_tasks(conn, project)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(
                    &rows,
                    &cli.tsv_config(&["id", "title", "tags", "startDate", "deadline"]),
                )?;
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn run_write_command(cli: &Cli, opener: &dyn UrlOpener) -> Result<(), Error> {
    match &cli.command {
        Command::Add {
            title,
            notes,
            when,
            deadline,
            tags,
            list,
            heading,
            checklist_items,
        } => {
            let url = write::build_add_url(&AddParams {
                title,
                notes: notes.as_deref(),
                when: when.as_deref(),
                deadline: deadline.as_deref(),
                tags: tags.as_deref(),
                list: list.as_deref(),
                heading: heading.as_deref(),
                checklist_items,
            });
            opener.open(&url)?;
            // r[cmd.add.output]
            println!("Created todo: {title}");
        }
        Command::Complete { id } => {
            let token = require_auth_token(cli)?;
            let url = write::build_complete_url(id, &token);
            opener.open(&url)?;
            println!("Completed: {id}");
        }
        Command::Cancel { id } => {
            let token = require_auth_token(cli)?;
            let url = write::build_cancel_url(id, &token);
            opener.open(&url)?;
            println!("Cancelled: {id}");
        }
        Command::Update {
            id,
            title,
            notes,
            append_notes,
            prepend_notes,
            when,
            deadline,
            add_tags,
            list,
            heading,
        } => {
            let token = require_auth_token(cli)?;
            let url = write::build_update_url(&UpdateParams {
                id,
                auth_token: &token,
                title: title.as_deref(),
                notes: notes.as_deref(),
                append_notes: append_notes.as_deref(),
                prepend_notes: prepend_notes.as_deref(),
                when: when.as_deref(),
                deadline: deadline.as_deref(),
                add_tags: add_tags.as_deref(),
                list: list.as_deref(),
                heading: heading.as_deref(),
            });
            opener.open(&url)?;
            println!("Updated: {id}");
        }
        Command::Move { id, to } => {
            let token = require_auth_token(cli)?;
            let url = write::build_move_url(id, &token, to);
            opener.open(&url)?;
            println!("Moved: {id}");
        }
        Command::Project {
            command:
                ProjectCommand::Add {
                    title,
                    notes,
                    when,
                    deadline,
                    tags,
                    area,
                    todos,
                },
        } => {
            let url = write::build_add_project_url(&AddProjectParams {
                title,
                notes: notes.as_deref(),
                when: when.as_deref(),
                deadline: deadline.as_deref(),
                tags: tags.as_deref(),
                area: area.as_deref(),
                todos,
            });
            opener.open(&url)?;
            // r[cmd.project.add.output]
            println!("Created project: {title}");
        }
        _ => unreachable!(),
    }
    Ok(())
}

// r[error.auth-missing]
fn require_auth_token(cli: &Cli) -> Result<String, Error> {
    cli.auth_token.clone().ok_or(Error::AuthMissing)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
