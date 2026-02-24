# plumb finish

Close the active session.

## Synopsis

```bash
plumb finish
```

## Description

Marks the active session as finished and clears the active session pointer. No
further commands can operate on this session after it is closed.

## Arguments

None.

## Options

None.

## Examples

```bash
plumb finish
# Session a1b2c3d4 finished.
```

## Behavior

1. Checks that no item is currently `in_progress`. If one is, **the command
   refuses and prints an error.** You must `plumb done` the in-progress item
   first.
2. Updates the session status to `finished` in `session.scb`.
3. Clears the `.plumb/active` file (removes the active session pointer).

The session's data (SCB files, snapshots) remains on disk under
`.plumb/sessions/<session_id>/`. It is not deleted.

## Notes

- **Refuses if in-progress.** `plumb finish` will not close a session with an
  `in_progress` item. This prevents accidentally abandoning work. Run
  `plumb done` first.
- It is valid to finish a session that still has `todo` items. The session
  closes and those items are left as-is. This lets you abandon a session
  gracefully when you decide not to refactor remaining files.
- After finishing, you may start a new session with `plumb start`.
- Session data is retained on disk. Plumb 0.1 does not provide a command to
  delete old sessions.

## See also

- [plumb start](./start.md) -- begin a new session.
- [plumb done](./done.md) -- mark in-progress item done before finishing.
- [Sessions](../concepts/sessions.md) -- session lifecycle.
- [Workspace](../concepts/workspace.md) -- where session data persists.
