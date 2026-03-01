// r[test.read-commands]
// r[cmd.inbox] r[cmd.inbox.columns]
// r[cmd.today] r[cmd.today.columns]
// r[cmd.upcoming] r[cmd.upcoming.columns]
// r[cmd.anytime] r[cmd.anytime.columns]
// r[cmd.someday] r[cmd.someday.columns]
// r[cmd.logbook] r[cmd.logbook.limit] r[cmd.logbook.columns]
// r[cmd.projects] r[cmd.projects.columns]
// r[cmd.areas] r[cmd.areas.columns]
// r[cmd.tags] r[cmd.tags.columns]
// r[cmd.show] r[cmd.show.output]
// r[cmd.search] r[cmd.search.columns]
// r[cmd.stats] r[cmd.stats.output]
// r[fields.tags] r[fields.project] r[fields.area]

use crate::dates;
use crate::error::Error;
use crate::model::{ChecklistItem, KeyValue, Row};
use rusqlite::Connection;
use serde_json::Value;

/// Helper: collect tags for a task UUID.
fn get_tags(conn: &Connection, task_uuid: &str) -> Result<String, Error> {
    let mut stmt = conn.prepare_cached(
        "SELECT TMTag.title FROM TMTaskTag \
         JOIN TMTag ON TMTaskTag.tags = TMTag.uuid \
         WHERE TMTaskTag.tasks = ? \
         ORDER BY TMTag.\"index\""
    )?;
    let tags: Vec<String> = stmt
        .query_map([task_uuid], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags.join(","))
}

/// Data fields extracted from a task query row.
struct TaskData {
    title: String,
    project_title: Option<String>,
    area_title: Option<String>,
    start_date: Option<i64>,
    deadline: Option<i64>,
    stop_date: Option<f64>,
    status: Option<i64>,
}

/// Build a Row from a task query result, resolving tags/project/area.
fn task_row(conn: &Connection, uuid: &str, data: TaskData) -> Result<Row, Error> {
    let mut row = Row::new();
    row.set("id", Value::String(uuid.to_string()));
    row.set("title", Value::String(data.title));

    let tags = get_tags(conn, uuid)?;
    row.set("tags", if tags.is_empty() { Value::Null } else { Value::String(tags) });

    row.set("project", match data.project_title {
        Some(ref p) => Value::String(p.clone()),
        None => Value::Null,
    });
    row.set("area", match data.area_title {
        Some(ref a) => Value::String(a.clone()),
        None => Value::Null,
    });
    row.set("startDate", match data.start_date.and_then(dates::decode_things_date) {
        Some(d) => Value::String(d),
        None => Value::Null,
    });
    row.set("deadline", match data.deadline.and_then(dates::decode_things_date) {
        Some(d) => Value::String(d),
        None => Value::Null,
    });
    row.set("completedDate", match data.stop_date {
        Some(ts) if ts > 0.0 => Value::String(dates::unix_timestamp_to_date(ts)),
        _ => Value::Null,
    });
    if let Some(s) = data.status {
        let status_str = match s {
            0 => "open",
            2 => "cancelled",
            3 => "completed",
            _ => "unknown",
        };
        row.set("status", Value::String(status_str.to_string()));
    }
    Ok(row)
}

const TASK_SELECT: &str =
    "SELECT t.uuid, t.title, \
     p.title AS projectTitle, \
     a.title AS areaTitle, \
     t.startDate, t.deadline, t.stopDate, t.status \
     FROM TMTask t \
     LEFT JOIN TMTask p ON t.project = p.uuid AND p.type = 1 \
     LEFT JOIN TMArea a ON t.area = a.uuid";

fn query_tasks(conn: &Connection, where_clause: &str, order: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<Vec<Row>, Error> {
    let sql = format!("{TASK_SELECT} WHERE {where_clause} ORDER BY {order}");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok((
            row.get::<_, String>(0)?,         // uuid
            row.get::<_, String>(1)?,         // title
            row.get::<_, Option<String>>(2)?, // projectTitle
            row.get::<_, Option<String>>(3)?, // areaTitle
            row.get::<_, Option<i64>>(4)?,    // startDate
            row.get::<_, Option<i64>>(5)?,    // deadline
            row.get::<_, Option<f64>>(6)?,    // stopDate
            row.get::<_, Option<i64>>(7)?,    // status
        ))
    })?;

    let mut result = Vec::new();
    for r in rows {
        let (uuid, title, proj, area, sd, dl, stop, status) = r?;
        result.push(task_row(conn, &uuid, TaskData {
            title, project_title: proj, area_title: area,
            start_date: sd, deadline: dl, stop_date: stop, status,
        })?);
    }
    Ok(result)
}

pub fn inbox(conn: &Connection) -> Result<Vec<Row>, Error> {
    query_tasks(
        conn,
        "t.start = 0 AND t.status = 0 AND t.trashed = 0 AND t.type = 0 \
         AND t.project IS NULL AND t.area IS NULL",
        "t.\"index\"",
        &[],
    )
}

pub fn today(conn: &Connection) -> Result<Vec<Row>, Error> {
    query_tasks(
        conn,
        "t.start = 1 AND t.status = 0 AND t.trashed = 0 AND t.type = 0",
        "t.todayIndex",
        &[],
    )
}

pub fn upcoming(conn: &Connection) -> Result<Vec<Row>, Error> {
    query_tasks(
        conn,
        "t.startDate IS NOT NULL AND t.startDate > 0 AND t.status = 0 AND t.trashed = 0 AND t.type = 0",
        "t.startDate ASC",
        &[],
    )
}

pub fn anytime(conn: &Connection) -> Result<Vec<Row>, Error> {
    query_tasks(
        conn,
        "t.start = 1 AND t.status = 0 AND t.trashed = 0 AND t.type = 0 \
         AND (t.todayIndex IS NULL OR t.todayIndex = 0)",
        "t.\"index\"",
        &[],
    )
}

pub fn someday(conn: &Connection) -> Result<Vec<Row>, Error> {
    query_tasks(
        conn,
        "t.start = 2 AND t.status = 0 AND t.trashed = 0 AND t.type = 0",
        "t.\"index\"",
        &[],
    )
}

pub fn logbook(conn: &Connection, limit: u32) -> Result<Vec<Row>, Error> {
    let sql = format!(
        "{TASK_SELECT} WHERE t.status = 3 AND t.trashed = 0 AND t.type = 0 \
         ORDER BY t.stopDate DESC LIMIT ?1"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<i64>>(5)?,
            row.get::<_, Option<f64>>(6)?,
            row.get::<_, Option<i64>>(7)?,
        ))
    })?;
    let mut result = Vec::new();
    for r in rows {
        let (uuid, title, proj, area, sd, dl, stop, status) = r?;
        result.push(task_row(conn, &uuid, TaskData {
            title, project_title: proj, area_title: area,
            start_date: sd, deadline: dl, stop_date: stop, status,
        })?);
    }
    Ok(result)
}

pub fn projects(conn: &Connection) -> Result<Vec<Row>, Error> {
    let sql =
        "SELECT t.uuid, t.title, a.title AS areaTitle, \
         t.deadline, t.openUntrashedLeafActionsCount \
         FROM TMTask t \
         LEFT JOIN TMArea a ON t.area = a.uuid \
         WHERE t.type = 1 AND t.status = 0 AND t.trashed = 0 \
         ORDER BY t.\"index\"";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<i64>>(4)?,
        ))
    })?;
    let mut result = Vec::new();
    for r in rows {
        let (uuid, title, area, deadline, open_tasks) = r?;
        let mut row = Row::new();
        row.set("id", Value::String(uuid.clone()));
        row.set("title", Value::String(title));
        row.set("area", match area {
            Some(a) => Value::String(a),
            None => Value::Null,
        });
        let tags = get_tags(conn, &uuid)?;
        row.set("tags", if tags.is_empty() { Value::Null } else { Value::String(tags) });
        row.set("deadline", match deadline.and_then(dates::decode_things_date) {
            Some(d) => Value::String(d),
            None => Value::Null,
        });
        row.set("openTasks", Value::Number(serde_json::Number::from(open_tasks.unwrap_or(0))));
        result.push(row);
    }
    Ok(result)
}

pub fn areas(conn: &Connection) -> Result<Vec<Row>, Error> {
    let mut stmt = conn.prepare(
        "SELECT uuid, title FROM TMArea WHERE visible = 1 ORDER BY \"index\""
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut result = Vec::new();
    for r in rows {
        let (uuid, title) = r?;
        let mut row = Row::new();
        row.set("id", Value::String(uuid));
        row.set("title", Value::String(title));
        result.push(row);
    }
    Ok(result)
}

pub fn tags(conn: &Connection) -> Result<Vec<Row>, Error> {
    let mut stmt = conn.prepare(
        "SELECT t.uuid, t.title, t.shortcut, p.title AS parentTitle \
         FROM TMTag t \
         LEFT JOIN TMTag p ON t.parent = p.uuid \
         ORDER BY t.\"index\""
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut result = Vec::new();
    for r in rows {
        let (uuid, title, shortcut, parent) = r?;
        let mut row = Row::new();
        row.set("id", Value::String(uuid));
        row.set("title", Value::String(title));
        row.set("shortcut", match shortcut {
            Some(s) if !s.is_empty() => Value::String(s),
            _ => Value::Null,
        });
        row.set("parent", match parent {
            Some(p) => Value::String(p),
            None => Value::Null,
        });
        result.push(row);
    }
    Ok(result)
}

pub fn show(conn: &Connection, id: &str) -> Result<Row, Error> {
    // Try as task first
    let sql = format!(
        "{TASK_SELECT} WHERE t.uuid = ?1"
    );
    let mut stmt = conn.prepare(&sql)?;
    let task = stmt.query_row([id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<i64>>(5)?,
            row.get::<_, Option<f64>>(6)?,
            row.get::<_, Option<i64>>(7)?,
        ))
    });

    match task {
        Ok((uuid, title, proj, area, sd, dl, stop, status)) => {
            let mut row = task_row(conn, &uuid, TaskData {
                title, project_title: proj, area_title: area,
                start_date: sd, deadline: dl, stop_date: stop, status,
            })?;

            // Add notes
            let notes: Option<String> = conn.query_row(
                "SELECT notes FROM TMTask WHERE uuid = ?1",
                [id],
                |r| r.get(0),
            )?;
            row.set("notes", match notes {
                Some(n) if !n.is_empty() => Value::String(n),
                _ => Value::Null,
            });

            // Add type
            let task_type: i64 = conn.query_row(
                "SELECT type FROM TMTask WHERE uuid = ?1",
                [id],
                |r| r.get(0),
            )?;
            row.set("type", Value::String(match task_type {
                0 => "task",
                1 => "project",
                2 => "heading",
                _ => "unknown",
            }.to_string()));

            // Add checklist items
            let mut cl_stmt = conn.prepare(
                "SELECT title, status FROM TMChecklistItem \
                 WHERE task = ?1 ORDER BY \"index\""
            )?;
            let items: Vec<ChecklistItem> = cl_stmt
                .query_map([id], |r| {
                    Ok(ChecklistItem {
                        title: r.get(0)?,
                        status: r.get(1)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();
            if !items.is_empty() {
                let arr: Vec<Value> = items
                    .iter()
                    .map(|i| {
                        serde_json::json!({
                            "title": i.title,
                            "completed": i.status == 3,
                        })
                    })
                    .collect();
                row.set("checklistItems", Value::Array(arr));
            }

            Ok(row)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Try as area
            let area_result: Result<(String, String), _> = conn.query_row(
                "SELECT uuid, title FROM TMArea WHERE uuid = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            );
            match area_result {
                Ok((uuid, title)) => {
                    let mut row = Row::new();
                    row.set("id", Value::String(uuid));
                    row.set("title", Value::String(title));
                    row.set("type", Value::String("area".to_string()));
                    Ok(row)
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound(id.to_string())),
                Err(e) => Err(Error::Sqlite(e)),
            }
        }
        Err(e) => Err(Error::Sqlite(e)),
    }
}

pub fn search(conn: &Connection, query: &str) -> Result<Vec<Row>, Error> {
    let pattern = format!("%{query}%");
    query_tasks(
        conn,
        "(t.title LIKE ?1 OR t.notes LIKE ?1) AND t.trashed = 0 AND (t.type = 0 OR t.type = 1)",
        "t.title",
        &[&pattern as &dyn rusqlite::types::ToSql],
    )
}

pub fn stats(conn: &Connection) -> Result<Vec<KeyValue>, Error> {
    let mut result = Vec::new();

    let counts = [
        ("inbox", "SELECT COUNT(*) FROM TMTask WHERE start = 0 AND status = 0 AND trashed = 0 AND type = 0 AND project IS NULL AND area IS NULL"),
        ("today", "SELECT COUNT(*) FROM TMTask WHERE start = 1 AND status = 0 AND trashed = 0 AND type = 0"),
        ("upcoming", "SELECT COUNT(*) FROM TMTask WHERE startDate IS NOT NULL AND startDate > 0 AND status = 0 AND trashed = 0 AND type = 0"),
        ("someday", "SELECT COUNT(*) FROM TMTask WHERE start = 2 AND status = 0 AND trashed = 0 AND type = 0"),
        ("completed", "SELECT COUNT(*) FROM TMTask WHERE status = 3 AND trashed = 0"),
        ("cancelled", "SELECT COUNT(*) FROM TMTask WHERE status = 2 AND trashed = 0"),
        ("trashed", "SELECT COUNT(*) FROM TMTask WHERE trashed = 1"),
        ("projects", "SELECT COUNT(*) FROM TMTask WHERE type = 1 AND status = 0 AND trashed = 0"),
        ("areas", "SELECT COUNT(*) FROM TMArea WHERE visible = 1"),
        ("tags", "SELECT COUNT(*) FROM TMTag"),
    ];

    for (label, sql) in counts {
        let count: i64 = conn.query_row(sql, [], |r| r.get(0))?;
        result.push(KeyValue {
            key: label.to_string(),
            value: Value::Number(serde_json::Number::from(count)),
        });
    }
    Ok(result)
}
