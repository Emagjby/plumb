# plumb(1) Manual

## What is Plumb

Plumb is a tiny CLI for running **refactor sessions** as a disciplined queue of
**files**. A session is a short-lived work container: you enqueue the files you
plan to touch, pick one, refactor it, prove the change with a diff against an
automatic baseline snapshot, mark it done, and move on. When the queue is empty
you close the session. That is the entire workflow.

---

## Quick start

```bash
# 1. Begin a session
plumb start "refactor auth guards"

# 2. Add individual files
plumb add src/auth/guard.rs
plumb add src/auth/session.rs

# 3. Or add an entire folder (recursive)
plumb add -f src/middleware/

# 4. Check what you queued
plumb status

# 5. Pick a file and start refactoring
plumb go 1                   # captures baseline, opens file in vim

# ... refactor in vim, then :wq ...

# 6. See what changed since go-time
plumb diff

# 7. Mark it done
plumb done

# 8. What's next?
plumb next

# 9. When the queue is empty, close the session
plumb finish
```

See a complete walkthrough: [Typical flow](./examples/typical-flow.md).

---

## Concepts

| Concept                              | Description                                                |
| ------------------------------------ | ---------------------------------------------------------- |
| [Sessions](./concepts/sessions.md)   | Short-lived work containers. One active session at a time. |
| [Items](./concepts/items.md)         | Files tracked inside a session, each with an integer ID.   |
| [States](./concepts/states.md)       | `todo` -> `in_progress` -> `done`.                         |
| [Snapshots](./concepts/snapshots.md) | Go-time baselines and the diff model.                      |
| [Workspace](./concepts/workspace.md) | `.plumb/` directory layout and Strata SCB storage.         |

---

## Commands

| Command                              | Description                                                   |
| ------------------------------------ | ------------------------------------------------------------- |
| [plumb start](./commands/start.md)   | Begin a new session.                                          |
| [plumb add](./commands/add.md)       | Enqueue files (or a folder) as todo items.                    |
| [plumb rm](./commands/rm.md)         | Remove an item from the queue.                                |
| [plumb status](./commands/status.md) | Show session progress and the current in-progress item.       |
| [plumb go](./commands/go.md)         | Start working on a specific item; captures baseline snapshot and opens vim. |
| [plumb diff](./commands/diff.md)     | Diff a target item, or all `in_progress` items when target is omitted. |
| [plumb done](./commands/done.md)         | Mark the in-progress item as done.                            |
| [plumb restore](./commands/restore.md)   | Restore a file to its baseline snapshot.                      |
| [plumb next](./commands/next.md)         | Print the next todo item without changing state.              |
| [plumb finish](./commands/finish.md) | Close the active session.                                     |

---

## Command summary (0.1)

- **`plumb start [name]`** -- Creates a new session and marks it active. Initialises `.plumb/` if it does not exist.
- **`plumb add <file>` / `plumb add -f <folder>`** -- Adds one file, or all files inside a folder recursively, as `todo` items in the active session.
- **`plumb rm <id|file>`** -- Removes an item from the session queue. Cannot remove an `in_progress` item.
- **`plumb status`** -- Prints counts of todo / in-progress / done items and the currently in-progress file, if any.
- **`plumb go <id|file>`** -- Sets one `todo` item to `in_progress`, captures a baseline snapshot of that file's current contents, and opens the file in vim.
- **`plumb diff [id|file]`** -- Shows the unified diff between go-time baseline and current contents. With no argument, diffs all `in_progress` items (no-op if none).
- **`plumb done [id|file]`** -- Moves the current `in_progress` item to `done`.
- **`plumb restore [id|file]`** -- Overwrites the file on disk with its go-time baseline snapshot. Prompts for confirmation before proceeding.
- **`plumb next`** -- Prints the next `todo` item (lowest ID) without changing any state.
- **`plumb finish`** -- Closes the active session. Refuses if any item is still `in_progress`.

---

## Storage

Plumb stores all persistent state under `.plumb/` at the workspace root.
Structured data is stored as **Strata SCB** binary files (`*.scb`). Strata SCB
is a deterministic, self-delimiting, language-neutral binary format -- every
value has exactly one canonical byte representation.

Baseline snapshots are stored as raw file bytes (not SCB-encoded).

See [Workspace](./concepts/workspace.md) for the full directory layout.

---

## Scope (0.1)

Plumb 0.1 is intentionally local and minimal:

- **No git integration.** Plumb does not read from or write to any VCS.
- **No AI integration.** No integration with AI models (In the future will be implemented with strict rules for LLMs, that force code review unless specified no.)
- **No multi-session dashboard.** One active session at a time; previous sessions are retained on disk but not queryable.
- **Single machine only.** `.plumb/` is a local directory, not synced anywhere.

Future versions may lift these constraints. Version 0.1 keeps the surface area
small so the core loop is solid.
