# plumb restore

Restore a file to its baseline snapshot.

## Synopsis

```
plumb restore [id|file]
```

## Description

Overwrites the file on disk with its **baseline snapshot** -- the exact bytes
captured at `plumb go` time. This reverts the file to the state it was in at the
moment you started working on it, discarding all changes made since.

Because this is a destructive operation, Plumb **prompts for confirmation**
before writing.

By default (no arguments), it restores the currently `in_progress` item.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `id` or `file` | No | The item to restore. Defaults to the current `in_progress` item. Accepts an integer ID or a file path relative to the workspace root. |

## Options

None.

## Confirmation prompt

Before overwriting the file, Plumb prints a warning and asks for confirmation:

```
Restore src/auth/guard.rs to baseline snapshot?
All changes since go-time will be lost.
Are you sure? [y/N]
```

- Typing `y` or `yes` (case-insensitive) proceeds with the restore.
- Any other input (including pressing Enter with no input) aborts.

If the user declines, the file is left untouched and Plumb exits with a message:

```
Restore cancelled.
```

## Behavior

When confirmed, `plumb restore` does the following:

1. Reads the baseline snapshot from
   `.plumb/sessions/<session_id>/snapshots/<item_id>.baseline`.
2. Overwrites the file on disk with those exact bytes.

The item's **state is not changed**. If the item was `in_progress`, it remains
`in_progress`. You can continue editing, run `plumb diff` (which will now show
an empty diff), or run `plumb done`.

## Examples

Restore the current in-progress item:

```bash
plumb restore
# Restore src/auth/guard.rs to baseline snapshot?
# All changes since go-time will be lost.
# Are you sure? [y/N] y
# Restored: [1] src/auth/guard.rs
```

Restore a specific item by ID:

```bash
plumb restore 3
# Restore src/middleware/cors.rs to baseline snapshot?
# All changes since go-time will be lost.
# Are you sure? [y/N] y
# Restored: [3] src/middleware/cors.rs
```

Decline the confirmation:

```bash
plumb restore
# Restore src/auth/guard.rs to baseline snapshot?
# All changes since go-time will be lost.
# Are you sure? [y/N] n
# Restore cancelled.
```

## Failure cases

| Scenario | Behavior |
|----------|----------|
| No argument and no item `in_progress` | Error: no in-progress item to restore. |
| Item is `todo` (no baseline exists) | Error: no baseline snapshot. Run `plumb go` first. |
| Baseline snapshot file is missing on disk | Error: baseline not found. This should not occur in normal use. |
| File path is not writable (permissions) | Error: cannot write to file. Baseline is not modified. |

## Notes

- **Destructive.** This overwrites the file on disk. There is no undo beyond
  whatever VCS you may be using outside of Plumb.
- **Confirmation required.** The `[y/N]` prompt defaults to "no" to prevent
  accidental data loss.
- **State unchanged.** Restoring does not move the item back to `todo` or
  forward to `done`. The item keeps its current state.
- You can restore `done` items too, as long as their baseline snapshot still
  exists on disk (snapshots are preserved until the session directory is cleaned
  up).
- After restoring, `plumb diff` will show an empty diff (file matches baseline).
- Requires an active session.

## See also

- [plumb go](./go.md) -- captures the baseline that restore writes back.
- [plumb diff](./diff.md) -- check what changed before deciding to restore.
- [plumb done](./done.md) -- mark the item done instead of restoring.
- [Snapshots](../concepts/snapshots.md) -- how baselines are stored.
