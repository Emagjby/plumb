# Milestone Plan — v0.1 (Plumb)

This document breaks **v0.1** into smaller implementation phases.
Each phase defines the outcome, “done when” criteria, and the tests to ship with it.

> Storage: **Strata SCB (`*.scb`)** for structured state under `.plumb/`.  
> Snapshots: **raw file bytes** captured at `plumb go` time.

---

## Phase 0.1.0 — Skeleton + Workspace + Active Session Pointer

### Outcome

The CLI runs, resolves a workspace, can start a session, and persists an “active session” pointer.

### Scope

- `plumb` binary scaffold (arg parsing + command routing)
- Workspace root resolution:
  - treat current dir as root if `.plumb/` absent
  - if `.plumb/` exists in a parent, use nearest parent as root
- `.plumb/` initialization on `start`
- Active session pointer:
  - `.plumb/active` stored as plain text session id (allowed for 0.1)
- Session directory creation:
  - `.plumb/sessions/<session_id>/`
  - empty `snapshots/` directory
- `session.scb` created with minimal metadata:
  - session id
  - optional name
  - created timestamp
  - status = active

### Done when

- `plumb start "name"` creates `.plumb/` layout and sets `.plumb/active`
- Starting when a session is already active fails with a clear error
- Running from a nested folder correctly detects the workspace root
- No panics; helpful messages; exit code non-zero on errors

### Tests

**Unit**

- workspace root resolution logic (path traversal)
- “start refuses if active session exists” decision logic

**Integration**

- temp dir workspace:
  - run `start`, assert `.plumb/active` exists and points to a session dir
  - assert session dir contains `session.scb` and `snapshots/`

**Regression**

- none required yet

---

## Phase 0.1.1 — Items SCB + `add` (single file)

### Outcome

You can enqueue files as items in `todo`, persisted in `items.scb` with stable IDs.

### Scope

- `items.scb` schema (v0.1 minimal):
  - list of items: { id, rel_path, state }
- Load/save items via a `store` module (commands never touch SCB directly)
- `plumb add <file>`:
  - normalize to workspace-relative path
  - append item with next id
  - default state `todo`
  - duplicate paths skipped with a notice

### Done when

- `plumb add` assigns IDs starting at 1 and increments sequentially
- Stored paths are relative to workspace root (not absolute)
- Duplicate add prints a notice and does not create a new item
- `plumb status` can at least print counts (even if minimal formatting)

### Tests

**Unit**

- id assignment from existing items
- path normalization (including `./` prefixes)
- duplicate detection by normalized relative path

**Integration**

- temp dir workspace:
  - create a couple files, `start`, then `add` them
  - assert items persisted and reloadable from `items.scb`

**Regression**

- none required yet

---

## Phase 0.1.1.2 — `rm` (remove item from queue)

### Outcome

You can remove items from the session queue by ID or file path, keeping the queue clean without restarting.

### Scope

- `plumb rm <id|file>`:
  - resolve item by integer ID or workspace-relative file path
  - normalize file path input the same way as `add` (relative to workspace root)
  - refuse to remove an `in_progress` item (must `done` or `restore` first)
  - allow removing `todo` and `done` items
  - delete baseline snapshot if one exists for the removed item
  - IDs are stable: removing an item does not renumber other items
  - persist updated item list to `items.scb`

### Done when

- `plumb rm 2` removes item 2 from the queue and prints confirmation
- `plumb rm src/auth/guard.rs` resolves path and removes the matching item
- Removing an `in_progress` item fails with a clear error
- Removing a non-existent ID or path fails with a clear error
- IDs of remaining items are unchanged after removal
- Baseline snapshot is cleaned up when a `done` item is removed
- `plumb status` no longer shows the removed item

### Tests

**Unit**

- removal by ID: item removed, others unchanged
- removal by path: path normalization + match
- refusal of in_progress removal
- ID stability after removal (no renumbering)
- error on non-existent ID / path

**Integration**

- temp dir workspace:
  - add 3 items, remove middle one, assert remaining items correct
  - add item, go, done, remove done item, assert baseline snapshot deleted
  - attempt to remove in_progress item, assert error

**Regression**

- none required yet

---

## Phase 0.1.2 — `add -f` folder expansion (recursive + deterministic)

### Outcome

You can bulk-enqueue a folder recursively, deterministically, without junk directories.

### Scope

- `plumb add -f <folder>`:
  - recursive walk
  - skip `.plumb/`, `.git/`, `node_modules/`, `target/` always
  - only add regular files
  - collect + sort lexicographically by workspace-relative path
  - assign IDs continuing from current max
  - skip duplicates with notices
- Document behavior already matches docs (no surprises)

### Done when

- `add -f` produces the same IDs/order across runs on the same tree
- Excludes are enforced
- Folder add persists items in one write (not per file)

### Tests

**Unit**

- folder collection + filtering + sorting (test with synthetic path lists)
- exclusion rules
- ID continuation

**Integration**

- temp dir workspace:
  - create nested dirs/files including excluded dirs
  - `add -f`, assert only expected files appear in items

**Regression**

- none required yet

---

## Phase 0.1.3 — `go` baseline capture + single in-progress rule

### Outcome

You can start an item: it becomes `in_progress` and a baseline snapshot is captured.

### Scope

- `plumb go <id|file>`:
  - resolve item by id or path
  - enforce single `in_progress`
  - require item state == `todo`
  - require file exists + readable
  - capture baseline to `snapshots/<item_id>.baseline` (raw bytes)
  - update item state to `in_progress`
  - best-effort editor open (optional; failure does not fail command)

### Done when

- `go` captures baseline bytes exactly equal to file bytes at go-time
- `go` refuses when another item is already `in_progress`
- `go` fails cleanly if file missing/unreadable and leaves item in `todo`
- `status` shows the current in-progress item

### Tests

**Unit**

- transition rules:
  - cannot go if any in_progress
  - cannot go a done item
  - baseline required only after go

**Integration**

- temp dir workspace:
  - create file with known content
  - `go`, assert snapshot file exists and matches bytes
  - attempt `go` another item, assert error

**Regression**

- none required yet

---

## Phase 0.1.4 — `diff` unified diff (baseline vs current)

### Outcome

You can view exactly what changed since go-time baseline.

### Scope

- `plumb diff [id|file]`:
  - default to current `in_progress` when no arg
  - load baseline bytes
  - load current bytes (if file missing: treat as empty)
  - render unified diff (portable Rust implementation preferred)
  - read-only command (no state changes)

### Done when

- After modifying a file post-go, `diff` shows a hunk containing the change
- If file unchanged, diff is empty (or prints “no changes” consistently)
- If file deleted, diff shows it as removed (baseline -> empty)
- Helpful error when baseline missing (item never started)

### Tests

**Unit**

- diff invocation rules (target resolution, baseline required)
- “missing current file treated as empty” logic

**Integration**

- temp dir workspace:
  - baseline capture, then modify file, assert diff contains expected line
  - delete file, assert diff indicates removal

**Regression**

- golden diff fixtures:
  - one-line modification
  - deletion case

---

## Phase 0.1.5 — `done`, `next`, `status` polish

### Outcome

The workflow feels usable: you can complete items, inspect queue, and pull the next file.

### Scope

- `plumb done [id|file]`:
  - default: current `in_progress`
  - only allowed when item is `in_progress`
- `plumb next`:
  - print lowest-id `todo`, no state change
- `plumb status`:
  - print counts and current in-progress
  - stable human output format

### Done when

- `done` only works for in-progress; errors otherwise
- `next` matches lowest-id todo (deterministic)
- `status` output is stable and includes key information

### Tests

**Unit**

- done transition rule
- next selection rule

**Integration**

- temp dir workflow:
  - add 3 items, go 1, done, next returns 2, etc.

**Regression**

- none required yet

---

## Phase 0.1.6 — `restore` + `finish` + crash-safe writes

### Outcome

You can safely revert changes and close sessions without corrupting state.

### Scope

- `plumb restore [id|file]`:
  - default: current `in_progress`
  - requires baseline snapshot exists
  - confirmation prompt `[y/N]`
  - overwrite file with baseline bytes
  - state unchanged
- `plumb finish`:
  - refuse if any item `in_progress`
  - mark session finished in `session.scb`
  - clear `.plumb/active`
- Crash-safe writes:
  - write temp + rename for SCB files and active pointer

### Done when

- `restore` rewrites the file exactly to baseline bytes (byte-for-byte)
- declining confirmation does nothing
- `finish` refuses when an item is in progress
- `finish` clears active and preserves session dir
- killing the process mid-write does not leave corrupted primary files (tmp may remain)

### Tests

**Unit**

- restore permission logic: baseline required, default target rules
- finish refusal when in_progress exists

**Integration**

- restore happy path: modify file, restore, assert bytes match baseline
- finish: ensure active cleared and session status updated

**Regression**

- confirmation prompt default behavior (`Enter` = no)
- optional: simulate interrupted write by ensuring temp-write strategy exists (structure test)

---

## Phase 0.1.7 — Docs + release hygiene (final pass)

### Outcome

v0.1 is shippable: docs match behavior, binary is robust, CI checks pass.

### Scope

- Ensure `/docs` matches actual behavior (editor optional, SCB usage, excludes)
- Ensure help text mirrors docs for commands/flags
- `cargo fmt`, `cargo clippy`, and tests passing
- Minimal versioning metadata in SCB (schema version or file version tag)

### Done when

- All integration tests pass on CI
- No undocumented behavior differences vs docs
- CLI errors are readable and consistent

### Tests

**Unit**

- none new (unless behavior drift found)

**Integration**

- one end-to-end “typical flow” integration test using temp workspace

**Regression**

- keep the golden diff fixtures stable
