# plumb add

Add files to the active session as `todo` items.

## Synopsis

```bash
plumb add <file>
plumb add -f <folder>
```

## Description

Enqueues one or more files into the active session. Each file becomes an
[item](../concepts/items.md) in `todo` state with a stable, auto-incrementing
integer ID.

There are two modes:

- **File mode** (default): adds a single file.
- **Folder mode** (`-f`): recursively adds all files inside the given directory.

## Arguments

| Argument | Required          | Description                                                   |
| -------- | ----------------- | ------------------------------------------------------------- |
| `file`   | Yes (file mode)   | Path to a single file to add.                                 |
| `folder` | Yes (folder mode) | Path to a directory whose contents will be added recursively. |

## Options

| Option           | Description                                                                |
| ---------------- | -------------------------------------------------------------------------- |
| `-f`, `--folder` | Treat the argument as a directory and recursively add all files inside it. |

## File mode

```bash
plumb add src/auth/guard.rs
```

Adds the file as a single `todo` item. The path is stored **relative to the
workspace root**.

## Folder mode

```bash
plumb add -f src/middleware/
```

Recursively walks the directory and adds every file found as a separate `todo`
item.

### Folder behavior

| Behavior                | Detail                                                                                                                                                                     |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Recursion**           | Descends into all subdirectories.                                                                                                                                          |
| **Directories skipped** | Only regular files are added; directories themselves are not items.                                                                                                        |
| **Default excludes**    | The following directories are always skipped: `.plumb/`, `.git/`, `node_modules/`, `target/`.                                                                              |
| **Ordering**            | Files are sorted **lexicographically by their path relative to the workspace root** before IDs are assigned. This produces deterministic, stable ordering across machines. |
| **Relative paths**      | All stored paths are relative to the workspace root.                                                                                                                       |

### Duplicate handling

If a file being added already exists in the session (same relative path), the
duplicate is **skipped** and a notice is printed:

```
notice: src/auth/guard.rs already in session, skipping.
```

The existing item retains its original ID and state.

### ID assignment

IDs continue incrementing from the session's current counter. If the session
already has items 1-5, the next added item receives ID 6 regardless of whether
it was added individually or as part of a folder.

## Examples

Add a single file:

```bash
plumb add src/auth/guard.rs
# Added: [1] src/auth/guard.rs
```

Add another file:

```bash
plumb add src/auth/session.rs
# Added: [2] src/auth/session.rs
```

Add an entire folder:

```bash
plumb add -f src/middleware/
# Added: [3] src/middleware/cors.rs
# Added: [4] src/middleware/logging.rs
# Added: [5] src/middleware/rate_limit.rs
```

Attempt to add a duplicate:

```bash
plumb add src/auth/guard.rs
# notice: src/auth/guard.rs already in session, skipping.
```

## Notes

- Requires an active session. Fails if no session is active.
- Paths are always stored relative to the workspace root.
- The file does not need to exist on disk at add-time (it must exist at
  `plumb go` time when the baseline is captured). However, implementations may
  choose to warn if the file is missing.
- Items are appended to `items.scb` (Strata SCB binary).

## See also

- [plumb go](./go.md) -- start working on an added item.
- [plumb status](./status.md) -- see all items and their states.
- [Items](../concepts/items.md) -- item fields and ID assignment.
- [Workspace](../concepts/workspace.md) -- where items are stored.
