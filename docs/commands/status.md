# plumb status

Show the state of the active session.

## Synopsis

```bash
plumb status
```

## Description

Prints a summary of the active session: counts of items in each state and the
file currently being worked on (if any).

## Arguments

None.

## Options

None.

## Output

The output includes:

- **Session name** (if one was given at start).
- **Counts** for each state: `todo`, `in_progress`, `done`.
- **Current item**: the file currently `in_progress`, if any.

Example output:

```
Session: refactor auth guards

  todo:        3
  in_progress: 1
  done:        2

In progress: [3] src/middleware/cors.rs
```

If no item is in progress:

```
Session: refactor auth guards

  todo:        3
  in_progress: 0
  done:        2

No item in progress.
```

## Examples

```bash
plumb status
```

## Notes

- Requires an active session. Fails if no session is active.
- This command is read-only. It does not change any item state.

## See also

- [plumb next](./next.md) -- print the next todo item.
- [plumb go](./go.md) -- start working on an item.
- [Sessions](../concepts/sessions.md)
- [States](../concepts/states.md)
