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
use write::{AddParams, MacOsUrlOpener, UpdateParams, UrlOpener};

#[derive(Parser)]
#[command(name = "tdo", about = "Things 3 CLI — machine-friendly interface")]
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
        if self.no_header { Header::Hide } else { Header::Show }
    }

    fn tsv_config<'a>(&'a self, default_fields: &'a [&'a str]) -> TsvConfig<'a> {
        TsvConfig {
            default_fields,
            fields: &self.fields,
            header: self.header(),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// List inbox items
    Inbox,
    /// List today items
    Today,
    /// List upcoming items
    Upcoming,
    /// List anytime items
    Anytime,
    /// List someday items
    Someday,
    /// List completed items
    Logbook {
        /// Maximum number of items to show
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// List open projects
    Projects,
    /// List areas
    Areas,
    /// List tags
    Tags,
    /// Show full details of an item
    Show {
        /// Item UUID
        id: String,
    },
    /// Search tasks by title/notes
    Search {
        /// Search query
        query: String,
    },
    /// Show database statistics
    Stats,
    /// Add a new todo
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
    /// Mark an item as completed
    Complete {
        /// Item UUID
        id: String,
    },
    /// Mark an item as cancelled
    Cancel {
        /// Item UUID
        id: String,
    },
    /// Update an existing item
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
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        // Read commands need DB access
        Command::Inbox | Command::Today | Command::Upcoming | Command::Anytime
        | Command::Someday | Command::Logbook { .. } | Command::Projects | Command::Areas
        | Command::Tags | Command::Show { .. } | Command::Search { .. } | Command::Stats => {
            let db_path = match &cli.db_path {
                Some(p) => p.clone(),
                None => db::find_db_path()?,
            };
            let conn = db::open_db(&db_path)?;
            run_read_command(&cli, &conn)
        }
        // Write commands use URL scheme
        Command::Add { .. } | Command::Complete { .. } | Command::Cancel { .. }
        | Command::Update { .. } | Command::Move { .. } => {
            run_write_command(&cli, &MacOsUrlOpener)
        }
    }
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
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "project", "tags", "deadline"]))?;
            }
        }
        Command::Upcoming => {
            let rows = queries::upcoming(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "project", "tags", "startDate", "deadline"]))?;
            }
        }
        Command::Anytime => {
            let rows = queries::anytime(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "project", "area", "tags", "deadline"]))?;
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
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "project", "completedDate"]))?;
            }
        }
        Command::Projects => {
            let rows = queries::projects(conn)?;
            if cli.json {
                output::print_json(&rows)?;
            } else {
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "area", "tags", "deadline", "openTasks"]))?;
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
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "shortcut", "parent"]))?;
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
                output::print_tsv(&rows, &cli.tsv_config(&["id", "title", "project", "status", "tags"]))?;
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
        _ => unreachable!(),
    }
    Ok(())
}

fn run_write_command(cli: &Cli, opener: &dyn UrlOpener) -> Result<(), Error> {
    match &cli.command {
        Command::Add {
            title, notes, when, deadline, tags, list, heading, checklist_items,
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
            id, title, notes, append_notes, prepend_notes,
            when, deadline, add_tags, list, heading,
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
