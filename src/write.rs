// r[data.write]
// r[data.write.auth]
// r[cmd.add] r[cmd.add.params] r[cmd.add.output]
// r[cmd.project.add] r[cmd.project.add.params] r[cmd.project.add.output]
// r[cmd.complete]
// r[cmd.cancel]
// r[cmd.update] r[cmd.update.params]
// r[cmd.move]

use crate::error::Error;

/// Trait for opening URLs, injectable for testing.
pub trait UrlOpener {
    fn open(&self, url: &str) -> Result<String, Error>;
}

/// Default URL opener that uses macOS `open` command.
pub struct MacOsUrlOpener;

impl UrlOpener for MacOsUrlOpener {
    fn open(&self, url: &str) -> Result<String, Error> {
        let output = std::process::Command::new("open").arg(url).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::Io(std::io::Error::other(format!(
                "Failed to open URL: {stderr}"
            ))))
        }
    }
}

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push_str(&format!("%{b:02X}"));
            }
        }
    }
    result
}

/// Parameters for adding a new todo.
pub struct AddParams<'a> {
    pub title: &'a str,
    pub notes: Option<&'a str>,
    pub when: Option<&'a str>,
    pub deadline: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub list: Option<&'a str>,
    pub heading: Option<&'a str>,
    pub checklist_items: &'a [String],
}

/// Parameters for updating an existing item.
pub struct UpdateParams<'a> {
    pub id: &'a str,
    pub auth_token: &'a str,
    pub title: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub append_notes: Option<&'a str>,
    pub prepend_notes: Option<&'a str>,
    pub when: Option<&'a str>,
    pub deadline: Option<&'a str>,
    pub add_tags: Option<&'a str>,
    pub list: Option<&'a str>,
    pub heading: Option<&'a str>,
}

/// Build a Things URL for adding a new todo.
pub fn build_add_url(params: &AddParams) -> String {
    let mut parts = vec![format!("title={}", url_encode(params.title))];

    if let Some(n) = params.notes {
        parts.push(format!("notes={}", url_encode(n)));
    }
    if let Some(w) = params.when {
        parts.push(format!("when={}", url_encode(w)));
    }
    if let Some(d) = params.deadline {
        parts.push(format!("deadline={}", url_encode(d)));
    }
    if let Some(t) = params.tags {
        parts.push(format!("tags={}", url_encode(t)));
    }
    if let Some(l) = params.list {
        parts.push(format!("list={}", url_encode(l)));
    }
    if let Some(h) = params.heading {
        parts.push(format!("heading={}", url_encode(h)));
    }
    if !params.checklist_items.is_empty() {
        let items = params.checklist_items.join("\n");
        parts.push(format!("checklist-items={}", url_encode(&items)));
    }

    format!("things:///add?{}", parts.join("&"))
}

/// Parameters for adding a new project.
pub struct AddProjectParams<'a> {
    pub title: &'a str,
    pub notes: Option<&'a str>,
    pub when: Option<&'a str>,
    pub deadline: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub area: Option<&'a str>,
    pub todos: &'a [String],
}

/// Build a Things URL for adding a new project.
pub fn build_add_project_url(params: &AddProjectParams) -> String {
    let mut parts = vec![format!("title={}", url_encode(params.title))];

    if let Some(n) = params.notes {
        parts.push(format!("notes={}", url_encode(n)));
    }
    if let Some(w) = params.when {
        parts.push(format!("when={}", url_encode(w)));
    }
    if let Some(d) = params.deadline {
        parts.push(format!("deadline={}", url_encode(d)));
    }
    if let Some(t) = params.tags {
        parts.push(format!("tags={}", url_encode(t)));
    }
    if let Some(a) = params.area {
        parts.push(format!("area={}", url_encode(a)));
    }
    if !params.todos.is_empty() {
        // Things URL scheme expects to-dos as a JSON array of strings or objects
        let json_todos: Vec<String> = params
            .todos
            .iter()
            .map(|t| serde_json::json!({"title": t}).to_string())
            .collect();
        let json_arr = format!("[{}]", json_todos.join(","));
        parts.push(format!("to-dos={}", url_encode(&json_arr)));
    }

    format!("things:///add-project?{}", parts.join("&"))
}

/// Build a Things URL for completing an item.
pub fn build_complete_url(id: &str, auth_token: &str) -> String {
    format!(
        "things:///update?id={}&completed=true&auth-token={}",
        url_encode(id),
        url_encode(auth_token)
    )
}

/// Build a Things URL for cancelling an item.
pub fn build_cancel_url(id: &str, auth_token: &str) -> String {
    format!(
        "things:///update?id={}&canceled=true&auth-token={}",
        url_encode(id),
        url_encode(auth_token)
    )
}

/// Build a Things URL for updating an item.
pub fn build_update_url(params: &UpdateParams) -> String {
    let mut parts = vec![
        format!("id={}", url_encode(params.id)),
        format!("auth-token={}", url_encode(params.auth_token)),
    ];

    if let Some(t) = params.title {
        parts.push(format!("title={}", url_encode(t)));
    }
    if let Some(n) = params.notes {
        parts.push(format!("notes={}", url_encode(n)));
    }
    if let Some(n) = params.append_notes {
        parts.push(format!("append-notes={}", url_encode(n)));
    }
    if let Some(n) = params.prepend_notes {
        parts.push(format!("prepend-notes={}", url_encode(n)));
    }
    if let Some(w) = params.when {
        parts.push(format!("when={}", url_encode(w)));
    }
    if let Some(d) = params.deadline {
        parts.push(format!("deadline={}", url_encode(d)));
    }
    if let Some(t) = params.add_tags {
        parts.push(format!("add-tags={}", url_encode(t)));
    }
    if let Some(l) = params.list {
        parts.push(format!("list={}", url_encode(l)));
    }
    if let Some(h) = params.heading {
        parts.push(format!("heading={}", url_encode(h)));
    }

    format!("things:///update?{}", parts.join("&"))
}

/// Build a Things URL for moving an item.
pub fn build_move_url(id: &str, auth_token: &str, list: &str) -> String {
    format!(
        "things:///update?id={}&list={}&auth-token={}",
        url_encode(id),
        url_encode(list),
        url_encode(auth_token)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[test.write-urls]

    #[test]
    fn test_build_add_url_simple() {
        let url = build_add_url(&AddParams {
            title: "Buy milk",
            notes: None,
            when: None,
            deadline: None,
            tags: None,
            list: None,
            heading: None,
            checklist_items: &[],
        });
        assert_eq!(url, "things:///add?title=Buy%20milk");
    }

    #[test]
    fn test_build_add_url_full() {
        let url = build_add_url(&AddParams {
            title: "Test task",
            notes: Some("Some notes"),
            when: Some("today"),
            deadline: Some("2024-12-25"),
            tags: Some("tag1,tag2"),
            list: Some("My Project"),
            heading: Some("A heading"),
            checklist_items: &["item 1".to_string(), "item 2".to_string()],
        });
        assert!(url.starts_with("things:///add?"));
        assert!(url.contains("title=Test%20task"));
        assert!(url.contains("notes=Some%20notes"));
        assert!(url.contains("when=today"));
        assert!(url.contains("deadline=2024-12-25"));
        assert!(url.contains("tags=tag1%2Ctag2"));
        assert!(url.contains("list=My%20Project"));
        assert!(url.contains("heading=A%20heading"));
        assert!(url.contains("checklist-items=item%201%0Aitem%202"));
    }

    #[test]
    fn test_build_complete_url() {
        let url = build_complete_url("abc-123", "token123");
        assert_eq!(
            url,
            "things:///update?id=abc-123&completed=true&auth-token=token123"
        );
    }

    #[test]
    fn test_build_cancel_url() {
        let url = build_cancel_url("abc-123", "token123");
        assert_eq!(
            url,
            "things:///update?id=abc-123&canceled=true&auth-token=token123"
        );
    }

    #[test]
    fn test_build_update_url() {
        let url = build_update_url(&UpdateParams {
            id: "abc-123",
            auth_token: "token123",
            title: Some("New title"),
            notes: None,
            append_notes: None,
            prepend_notes: None,
            when: Some("tomorrow"),
            deadline: None,
            add_tags: Some("urgent"),
            list: None,
            heading: None,
        });
        assert!(url.contains("id=abc-123"));
        assert!(url.contains("auth-token=token123"));
        assert!(url.contains("title=New%20title"));
        assert!(url.contains("when=tomorrow"));
        assert!(url.contains("add-tags=urgent"));
    }

    #[test]
    fn test_build_add_project_url_simple() {
        let url = build_add_project_url(&AddProjectParams {
            title: "My Project",
            notes: None,
            when: None,
            deadline: None,
            tags: None,
            area: None,
            todos: &[],
        });
        assert_eq!(url, "things:///add-project?title=My%20Project");
    }

    #[test]
    fn test_build_add_project_url_full() {
        let url = build_add_project_url(&AddProjectParams {
            title: "Sprint 13",
            notes: Some("Sprint notes"),
            when: Some("today"),
            deadline: Some("2025-07-01"),
            tags: Some("work,sprint"),
            area: Some("Work"),
            todos: &["Task 1".to_string(), "Task 2".to_string()],
        });
        assert!(url.starts_with("things:///add-project?"));
        assert!(url.contains("title=Sprint%2013"));
        assert!(url.contains("notes=Sprint%20notes"));
        assert!(url.contains("when=today"));
        assert!(url.contains("deadline=2025-07-01"));
        assert!(url.contains("tags=work%2Csprint"));
        assert!(url.contains("area=Work"));
        assert!(url.contains("to-dos="));
    }

    #[test]
    fn test_build_move_url() {
        let url = build_move_url("abc-123", "token123", "My Project");
        assert!(url.contains("id=abc-123"));
        assert!(url.contains("list=My%20Project"));
        assert!(url.contains("auth-token=token123"));
    }
}
