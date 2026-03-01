# Things 3 SQLite Schema Reference

Database location: `~/Library/Group Containers/JLMPQHK86H.com.culturedcode.ThingsMac/ThingsData-*/Things Database.thingsdatabase/main.sqlite`

## Date Encoding

Integer date fields (`startDate`, `deadline`, `todayIndexReferenceDate`) use a bit-packed format:

```
Bits 16-26: year  (11 bits, >> 16)
Bits 12-15: month (4 bits,  >> 12 & 0xF)
Bits  7-11: day   (5 bits,  >> 7  & 0x1F)
Bits  0-6:  unused (always 0)
```

To encode: `(year << 16) | (month << 12) | (day << 7)`
To decode: `year = v >> 16`, `month = (v >> 12) & 0xF`, `day = (v >> 7) & 0x1F`

Example: `132541952` → `2022-06-28`

REAL date fields (`creationDate`, `userModificationDate`, `stopDate`) are Unix timestamps.

## TMTask

Primary table for tasks and projects.

| Column | Type | Notes |
|--------|------|-------|
| uuid | TEXT PK | UUID identifier |
| type | INTEGER | 0=task, 1=project, 2=heading |
| status | INTEGER | 0=open, 3=completed, 2=cancelled |
| trashed | INTEGER | 0=not trashed, 1=trashed |
| start | INTEGER | 0=inbox/no start, 1=today/started, 2=someday |
| startDate | INTEGER | Bit-packed date (see above) |
| deadline | INTEGER | Bit-packed date |
| title | TEXT | Item title |
| notes | TEXT | Markdown notes |
| project | TEXT | UUID of parent project (self-join TMTask type=1) |
| area | TEXT | UUID of parent area (join TMArea) |
| heading | TEXT | UUID of heading within project |
| todayIndex | INTEGER | Sort order in Today view |
| todayIndexReferenceDate | INTEGER | Bit-packed date for today index validity |
| stopDate | REAL | Unix timestamp when completed/cancelled |
| creationDate | REAL | Unix timestamp |
| userModificationDate | REAL | Unix timestamp |
| index | INTEGER | Sort order |
| openUntrashedLeafActionsCount | INTEGER | Count of open child tasks (for projects) |
| checklistItemsCount | INTEGER | Total checklist items |
| openChecklistItemsCount | INTEGER | Open checklist items |

## TMArea

| Column | Type | Notes |
|--------|------|-------|
| uuid | TEXT PK | UUID identifier |
| title | TEXT | Area name |
| visible | INTEGER | Visibility flag |
| index | INTEGER | Sort order |

## TMTag

| Column | Type | Notes |
|--------|------|-------|
| uuid | TEXT PK | UUID identifier |
| title | TEXT | Tag name |
| shortcut | TEXT | Keyboard shortcut |
| parent | TEXT | UUID of parent tag (for nested tags) |
| index | INTEGER | Sort order |

## TMTaskTag

Junction table for task-tag relationships.

| Column | Type | Notes |
|--------|------|-------|
| tasks | TEXT | UUID of task |
| tags | TEXT | UUID of tag |

## TMChecklistItem

| Column | Type | Notes |
|--------|------|-------|
| uuid | TEXT PK | UUID identifier |
| title | TEXT | Item text |
| status | INTEGER | 0=open, 3=completed |
| task | TEXT | UUID of parent task |
| index | INTEGER | Sort order |
| stopDate | REAL | Unix timestamp when completed |
| creationDate | REAL | Unix timestamp |

## TMSettings

| Column | Type | Notes |
|--------|------|-------|
| uuid | TEXT PK | |
| uriSchemeAuthenticationToken | TEXT | Auth token for URL scheme |

## TMAreaTag

Junction table for area-tag relationships (same structure as TMTaskTag but with `areas` column).
