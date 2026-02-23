# plumb start

Begin a new refactor session.

## Synopsis

```
plumb start [name]
```

## Description

Creates a new session and marks it as the **active session**. All subsequent
commands (`add`, `go`, `status`, etc.) operate on this session until it is
closed with `plumb finish`.

If the `.plumb/` directory does not exist, `plumb start` creates it along with
the required subdirectory structure.

## Arguments

| Argument | Required | Description                                                                                                                              |
| -------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `name`   | No       | Optional human-readable label for the session (e.g. `"refactor auth guards"`). Stored in session metadata but not used as an identifier. |

## Options

None.

## What gets created

On a fresh workspace (no `.plumb/` directory):

```
.plumb/
  active                                  # set to the new session ID
  sessions/
    <session_id>/
      session.scb                         # session metadata (Strata SCB binary)
      items.scb                           # empty item list (Strata SCB binary)
      snapshots/                          # empty directory
```

On an existing workspace, only the new session directory and the `active`
pointer are created/updated.

## Examples

Start a session with a name:

```bash
plumb start "refactor auth guards"
# Session a1b2c3d4 started.
```

Start a session without a name:

```bash
plumb start
# Session f5e6d7c8 started.
```

## Notes

- **One active session at a time.** If a session is already active,
  `plumb start` fails with an error. Run `plumb finish` first.
- The session ID is generated automatically and is unique.
- `session.scb` and `items.scb` are Strata SCB binary files. They are not
  human-readable.

## See also

- [plumb finish](./finish.md) -- close the active session.
- [Sessions](../concepts/sessions.md) -- what a session is.
- [Workspace](../concepts/workspace.md) -- `.plumb/` layout.
