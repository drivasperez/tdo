# tdo Guide and Functionality Gaps

## Guide Gaps

### 1. Project Creation Requirements Not Clear
**Issue**: The guide shows `tdo add "task" --list "ProjectName"` without clarifying that the project must already exist.

**Current text**:
```
--list (project name or ID)
```

**Should clarify**:
- Projects cannot be created via CLI
- The `--list` flag requires the project to already exist in Things 3
- Creating a task with `--list` pointing to a non-existent project will add the task to Inbox, not create the project

**Suggested addition**:
> Note: Projects can only be created in the Things 3 UI. The `--list` parameter requires an existing project name or ID. If the project doesn't exist, the task will be added to Inbox instead.

---

### 2. `tdo move --to` Parameter Accepts Both Name and ID, But Name is Strongly Preferred
**Issue**: The help text says "Target project or area" without indicating:
- Whether to use project name or ID
- That using project ID may silently fail or behave unexpectedly
- That project name is the correct/preferred approach

**Current text**:
```
--to <TO>                  Target project or area
```

**Observed behavior**:
- `tdo move <id> --to "ProjectName"` ✓ Works
- `tdo move <id> --to "<ProjectID>"` ✗ Reports "Moved" but doesn't actually move

**Suggested addition**:
> Use the project name (not ID) with `--to`. Example: `tdo move ABC-123 --to "My Project"`. Using the project ID may report success but fail silently.

---

### 3. Missing Workflow Section
**Issue**: The guide documents individual commands but not common patterns for using them together.

**Missing workflows**:
- Creating multiple tasks in a project (with caveat that project must exist)
- Moving multiple tasks to a project (with ID lookup and batch operations)
- Typical job sequence (create project → create tasks → verify → assign to project)

**Suggested addition**: Add a "Common Workflows" section with examples like:

```
## Common Workflows

### Create Tasks in an Existing Project
Projects must be created in Things 3 first. Once a project exists:

tdo add "Task 1" --list "My Project"
tdo add "Task 2" --list "My Project"
tdo add "Task 3" --list "My Project" --heading "Section 1"
```

### Move Multiple Tasks to a Project
```
# Get task IDs, then move them
tdo search "keyword" --json | jq -r '.[].id' | while read id; do
  tdo move "$id" --to "Target Project"
done
```

---

### 4. Write Operations Use URL Scheme with Potential Delays
**Issue**: The guide mentions "Write commands use the Things URL scheme" but doesn't explain implications.

**Missing context**:
- Operations may not be instantaneous; database updates might lag
- Verification via `tdo search` might not show updated fields immediately
- The Things app briefly opens during write operations

**Suggested addition**:
> Write operations use the Things URL scheme and may involve brief delays before the database updates. To verify a write operation succeeded, use `tdo search` to query the updated item or check Things 3 directly.

---

### 5. `tdo move` Success Output is Misleading
**Issue**: The command prints "Moved: <id>" regardless of whether the move actually succeeded.

**Observed**: Using project ID instead of name prints "Moved" but doesn't actually move the task.

**Suggested addition**:
> To verify a move operation succeeded, search for the task and check that the `project` field is updated: `tdo search "title" --json`

---

### 6. Missing "Limitations" Section
**Issue**: The guide doesn't clearly document what operations are read-only vs. write-capable.

**Missing**: A clear statement that:
- **Read-only**: Projects can only be queried, not created/edited/deleted
- **Write capability for tasks**: Can create, complete, cancel, update, move
- **Write capability for projects**: None (project management is UI-only)

**Suggested addition**: Add a "Limitations" section:

```
## Limitations

### Project Management
- Projects can only be **created and modified in the Things 3 UI**
- The CLI can only **read** project information
- New projects cannot be created programmatically
- Project names, notes, and deadlines cannot be modified via CLI
- Projects cannot be deleted via CLI

### What You Can Do With Projects
- List projects: `tdo projects`
- View project details: `tdo show <project-id>`
- Assign new tasks to existing projects: `tdo add --list "ProjectName"`
- Move tasks between projects: `tdo move <id> --to "ProjectName"`

### What You Cannot Do
- Create projects programmatically
- Update project properties (name, deadline, etc.)
- Delete/archive projects
- Organize projects into areas (must be done in UI)
```

---

## Functionality Gaps

### 1. No Project Creation via CLI
**Gap**: Users cannot create projects programmatically. This breaks common automation patterns where you want to programmatically create both the project and its tasks.

**Impact**: In our workflow, we had to manually create the "Clean the flat" project in Things 3 before we could assign tasks to it.

**Potential solution**: Add a `tdo add-project` or equivalent command.

---

### 2. No Project Property Management via CLI
**Gap**: Cannot modify project names, notes, deadlines, or other properties programmatically.

**Impact**: If you need to batch-create projects or update them in bulk, you must do it in the UI.

**Potential solution**: Add `tdo update-project` command with flags for common properties.

---

### 3. No Project Deletion/Archival via CLI
**Gap**: Cannot delete or archive projects programmatically.

**Impact**: Cleanup and maintenance tasks require UI interaction.

---

### 4. Silent Failures with Wrong Parameter Types
**Gap**: Using a project ID with `tdo move --to` reports success but fails silently.

**Better behavior**: Should either:
- Accept both name and ID and work correctly with both
- Reject the ID and provide a clear error message
- Document in help text which format is expected

**Impact**: False confidence in operation success, requiring additional verification steps.

---

### 5. No Batch Move Command
**Gap**: Moving multiple tasks to a project requires looping; there's no batch operation.

**Current workaround**:
```bash
tdo search "keyword" | tail -n +2 | cut -f1 | while read id; do
  tdo move "$id" --to "ProjectName"
done
```

**Potential solution**: Add a batch flag like `tdo move --ids <id1,id2,id3> --to "ProjectName"` or accept stdin.

---

### 6. No Way to List Tasks by Project
**Gap**: While you can list projects and see `openTasks` count, there's no direct command to list all tasks in a specific project.

**Current workaround**: `tdo search "" --json | jq '.[] | select(.project == "ProjectName")'`

**Potential solution**: Add a `tdo project-tasks <project-name>` command or a filter flag like `tdo anytime --project "ProjectName"`.

---

### 7. Database Update Lag and Verification
**Gap**: No built-in way to ensure write operations completed before querying.

**Current workaround**: Add `sleep` commands between operations, or manually verify in Things 3.

**Better behavior**: Provide a `--wait` or `--verify` flag for write commands that doesn't return until the database reflects the change.

---

### 8. Limited Auth Token Management
**Gap**: Auth token must be set in environment or passed per-command; no convenience method for token persistence or retrieval.

**Current workaround**: Manual setup in `~/.config.local.fish` (or equivalent shell config).

**Potential solution**: A `tdo auth` command that manages token setup, or better documentation of the standard approach.

---

## Summary of Priority Fixes

**High Priority** (would improve usability significantly):
1. Document that projects cannot be created via CLI
2. Clarify that `--to` requires project name, not ID
3. Add "Common Workflows" section with examples
4. Implement a way to list tasks in a specific project

**Medium Priority** (would eliminate common workarounds):
1. Add project creation via CLI
2. Add batch move operation
3. Better error messages for invalid parameters

**Low Priority** (nice-to-have):
1. Project update/delete commands
2. Automated verification for write operations
3. Auth token management helpers
