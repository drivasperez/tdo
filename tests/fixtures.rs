use std::path::PathBuf;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct FixtureDb {
    path: PathBuf,
}

impl FixtureDb {
    pub fn path(&self) -> &str {
        self.path.to_str().unwrap()
    }
}

impl Drop for FixtureDb {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn create_fixture_db() -> FixtureDb {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir();
    let path = dir.join(format!("tdo-test-{}-{id}.sqlite", std::process::id()));

    let conn = rusqlite::Connection::open(&path).expect("failed to create fixture db");

    // Create schema
    conn.execute_batch("
        CREATE TABLE TMArea (
            uuid TEXT PRIMARY KEY,
            title TEXT,
            visible INTEGER,
            'index' INTEGER,
            cachedTags BLOB,
            experimental BLOB
        );

        CREATE TABLE TMTag (
            uuid TEXT PRIMARY KEY,
            title TEXT,
            shortcut TEXT,
            usedDate REAL,
            parent TEXT,
            'index' INTEGER,
            experimental BLOB
        );

        CREATE TABLE TMTaskTag (
            tasks TEXT NOT NULL,
            tags TEXT NOT NULL
        );
        CREATE INDEX index_TMTaskTag_tasks ON TMTaskTag(tasks);

        CREATE TABLE TMChecklistItem (
            uuid TEXT PRIMARY KEY,
            userModificationDate REAL,
            creationDate REAL,
            title TEXT,
            status INTEGER,
            stopDate REAL,
            'index' INTEGER,
            task TEXT,
            leavesTombstone INTEGER,
            experimental BLOB
        );
        CREATE INDEX index_TMChecklistItem_task ON TMChecklistItem(task);

        CREATE TABLE TMTask (
            uuid TEXT PRIMARY KEY,
            leavesTombstone INTEGER,
            creationDate REAL,
            userModificationDate REAL,
            type INTEGER,
            status INTEGER,
            stopDate REAL,
            trashed INTEGER,
            title TEXT,
            notes TEXT,
            notesSync INTEGER,
            cachedTags BLOB,
            start INTEGER,
            startDate INTEGER,
            startBucket INTEGER,
            reminderTime INTEGER,
            lastReminderInteractionDate REAL,
            deadline INTEGER,
            deadlineSuppressionDate INTEGER,
            t2_deadlineOffset INTEGER,
            'index' INTEGER,
            todayIndex INTEGER,
            todayIndexReferenceDate INTEGER,
            area TEXT,
            project TEXT,
            heading TEXT,
            contact TEXT,
            untrashedLeafActionsCount INTEGER,
            openUntrashedLeafActionsCount INTEGER,
            checklistItemsCount INTEGER,
            openChecklistItemsCount INTEGER,
            rt1_repeatingTemplate TEXT,
            rt1_recurrenceRule BLOB,
            rt1_instanceCreationStartDate INTEGER,
            rt1_instanceCreationPaused INTEGER,
            rt1_instanceCreationCount INTEGER,
            rt1_afterCompletionReferenceDate INTEGER,
            rt1_nextInstanceStartDate INTEGER,
            experimental BLOB,
            repeater BLOB,
            repeaterMigrationDate REAL
        );

        CREATE TABLE TMSettings (
            uuid TEXT PRIMARY KEY,
            logInterval INTEGER,
            manualLogDate REAL,
            groupTodayByParent INTEGER,
            uriSchemeAuthenticationToken TEXT,
            experimental BLOB
        );
    ").expect("failed to create schema");

    // Insert areas
    conn.execute(
        "INSERT INTO TMArea (uuid, title, visible, 'index') VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["area-work-1", "Work", 1, 0],
    ).unwrap();

    // Insert tags
    conn.execute(
        "INSERT INTO TMTag (uuid, title, shortcut, parent, 'index') VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["tag-urgent-1", "urgent", "u", rusqlite::types::Null, 0],
    ).unwrap();
    conn.execute(
        "INSERT INTO TMTag (uuid, title, shortcut, parent, 'index') VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["tag-home-1", "home", rusqlite::types::Null, rusqlite::types::Null, 1],
    ).unwrap();

    // Insert a project (type=1)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index', area, \
         openUntrashedLeafActionsCount) \
         VALUES (?1, 1, 0, 0, ?2, 1, 0, ?3, 2)",
        rusqlite::params!["project-test-1", "Test Project", "area-work-1"],
    ).unwrap();

    // Insert inbox task (start=0, no project, no area)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 0, 0)",
        rusqlite::params!["task-inbox-1", "Inbox task"],
    ).unwrap();

    // Insert today task (start=1)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, todayIndex, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 1, 1, 1)",
        rusqlite::params!["task-today-1", "Today task"],
    ).unwrap();

    // Insert today task with tag
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, todayIndex, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 1, 2, 2)",
        rusqlite::params!["task-today-tagged-1", "Today tagged task"],
    ).unwrap();
    conn.execute(
        "INSERT INTO TMTaskTag (tasks, tags) VALUES (?1, ?2)",
        rusqlite::params!["task-today-tagged-1", "tag-urgent-1"],
    ).unwrap();

    // Insert today task in project
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, todayIndex, 'index', project) \
         VALUES (?1, 0, 0, 0, ?2, 1, 3, 3, ?3)",
        rusqlite::params!["task-today-project-1", "Today project task", "project-test-1"],
    ).unwrap();

    // Insert upcoming task (has future startDate)
    // encode 2025-12-15: (2025 << 16) | (12 << 12) | (15 << 7)
    let upcoming_date = (2025_i64 << 16) | (12 << 12) | (15 << 7);
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, startDate, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 1, ?3, 4)",
        rusqlite::params!["task-upcoming-1", "Upcoming task", upcoming_date],
    ).unwrap();

    // Insert anytime task (start=1 but no todayIndex)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 1, 5)",
        rusqlite::params!["task-anytime-1", "Anytime task"],
    ).unwrap();

    // Insert someday task (start=2)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 2, 6)",
        rusqlite::params!["task-someday-1", "Someday task"],
    ).unwrap();

    // Insert completed task (status=3)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index', stopDate) \
         VALUES (?1, 0, 3, 0, ?2, 1, 7, ?3)",
        rusqlite::params!["task-completed-1", "Completed task", 1700000000.0_f64],
    ).unwrap();

    // Insert second completed task (older)
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index', stopDate) \
         VALUES (?1, 0, 3, 0, ?2, 1, 8, ?3)",
        rusqlite::params!["task-completed-2", "Older completed task", 1690000000.0_f64],
    ).unwrap();

    // Insert task with checklist items
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, 'index', \
         checklistItemsCount, openChecklistItemsCount) \
         VALUES (?1, 0, 0, 0, ?2, 1, 9, 2, 1)",
        rusqlite::params!["task-checklist-1", "Task with checklist"],
    ).unwrap();
    conn.execute(
        "INSERT INTO TMChecklistItem (uuid, title, status, task, 'index') VALUES (?1, ?2, 0, ?3, 0)",
        rusqlite::params!["cl-1", "Step 1", "task-checklist-1"],
    ).unwrap();
    conn.execute(
        "INSERT INTO TMChecklistItem (uuid, title, status, task, 'index') VALUES (?1, ?2, 3, ?3, 1)",
        rusqlite::params!["cl-2", "Step 2", "task-checklist-1"],
    ).unwrap();

    // Insert task with deadline
    let deadline_date = (2025_i64 << 16) | (6 << 12) | (30 << 7);
    conn.execute(
        "INSERT INTO TMTask (uuid, type, status, trashed, title, start, deadline, 'index') \
         VALUES (?1, 0, 0, 0, ?2, 0, ?3, 10)",
        rusqlite::params!["task-deadline-1", "Task with deadline", deadline_date],
    ).unwrap();

    FixtureDb { path }
}
