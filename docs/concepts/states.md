# States

Every [item](./items.md) in a session is in exactly one of three states.

## State definitions

| State | Meaning |
|-------|---------|
| `todo` | Queued for refactoring. Not yet started. |
| `in_progress` | Actively being refactored. A baseline snapshot exists. |
| `done` | Refactoring complete. |

## Transitions

```
todo ──> in_progress ──> done
```

- **`todo` -> `in_progress`** -- Triggered by `plumb go <id|file>`. A baseline
  snapshot of the file is captured at this moment.
- **`in_progress` -> `done`** -- Triggered by `plumb done [id|file]`.

No other transitions are valid. An item cannot move backwards (e.g. from `done`
back to `todo`). An item cannot skip states (e.g. from `todo` directly to
`done`).

## Single in-progress rule

At most **one item** may be `in_progress` at any time within a session. Running
`plumb go` when another item is already `in_progress` is an error -- you must
`plumb done` the current item first.

This constraint keeps the workflow linear and predictable: one file at a time.

## Checking state

- `plumb status` shows counts per state and the current `in_progress` item.
- `plumb next` shows the next `todo` item (lowest ID).

## See also

- [Items](./items.md) -- the objects that carry state.
- [Snapshots](./snapshots.md) -- baselines created on the `todo` -> `in_progress` transition.
- [plumb go](../commands/go.md)
- [plumb done](../commands/done.md)
- [plumb status](../commands/status.md)
