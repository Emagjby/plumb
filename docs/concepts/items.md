# Items

An **item** is a file tracked inside a session. When you `plumb add` a file, it
becomes an item in the active session's queue.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Stable, auto-incrementing identifier within the session. Starts at 1. |
| `path` | string | File path relative to the workspace root. |
| `state` | enum | One of `todo`, `in_progress`, or `done`. See [States](./states.md). |
| `baseline` | reference | Points to the baseline snapshot file, if one exists. Set when `plumb go` is run. |

## ID assignment

IDs are assigned sequentially in the order items are added. The counter never
resets within a session, even if items are added across multiple `plumb add`
invocations. For folder adds (`plumb add -f`), files are sorted
lexicographically by their path relative to the workspace root before IDs are
assigned, ensuring deterministic ordering.

## Duplicate handling

If you add a file that already exists in the session (same relative path), the
duplicate is **skipped** and a notice is printed. The existing item retains its
original ID and state.

## Storage

All items for a session are persisted in a single Strata SCB file:

```
.plumb/sessions/<session_id>/items.scb
```

## See also

- [States](./states.md) -- the lifecycle of an item.
- [Snapshots](./snapshots.md) -- baseline captures tied to items.
- [Sessions](./sessions.md) -- the container that holds items.
- [plumb add](../commands/add.md)
- [plumb go](../commands/go.md)
