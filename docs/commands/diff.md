# plumb diff

Show changes since the go-time baseline.

## Synopsis

```bash
plumb diff [id|file]
```

## Description

Produces a unified diff between the **baseline snapshot** (captured at
`plumb go` time) and the file's **current contents on disk**. This shows
exactly what you changed during your refactoring pass.

If you provide a target (`id` or `file`), Plumb diffs that specific item.

If you omit the target, Plumb diffs **all items currently in `in_progress`**.
If none are `in_progress`, the command succeeds and prints nothing.

## Arguments

| Argument       | Required | Description                                                                                                                        |
| -------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `id` or `file` | No       | The item to diff. If omitted, Plumb diffs all `in_progress` items. Accepts an integer ID or a file path relative to the workspace root. |

## Options

None.

## Diff model

```
baseline snapshot (raw bytes from go-time)  vs.  current file on disk
```

The output is a standard unified diff. There is no staging area, no three-way
merge, and no git involvement.

## Examples

Diff all currently in-progress items:

```bash
plumb diff
# --- baseline: src/auth/guard.rs
# +++ current:  src/auth/guard.rs
# @@ -10,7 +10,7 @@
# ...
# --- baseline: src/auth/session.rs
# +++ current:  src/auth/session.rs
# ...
```

Diff a specific item by ID:

```bash
plumb diff 2
```

Diff a specific item by path:

```bash
plumb diff src/auth/session.rs
```

## Failure cases

| Scenario                                      | Behavior                                                        |
| --------------------------------------------- | --------------------------------------------------------------- |
| No argument and no item `in_progress`         | Success with empty output (no-op).                              |
| Specified item is `todo` (no baseline exists) | Error: no baseline snapshot. Run `plumb go` first.              |
| Baseline snapshot file is missing on disk     | Error: baseline not found. This should not occur in normal use. |
| File deleted after go-time                    | Diff shows all lines removed (file treated as empty).           |
| Baseline or current file is non-UTF-8 bytes   | Prints `Binary files differ: <path>` for that item.             |
| File unchanged since go-time                  | Empty diff (no output).                                         |

## Notes

- **Read-only.** Does not change item state.
- The baseline is the raw bytes captured at `plumb go` time. See
  [Snapshots](../concepts/snapshots.md).
- You can diff `done` items if their baseline snapshot still exists on disk
  (it is preserved until the session directory is cleaned up).
- Requires an active session.

## See also

- [plumb go](./go.md) -- captures the baseline that diff compares against.
- [plumb restore](./restore.md) -- revert the file to the baseline if unhappy with changes.
- [plumb done](./done.md) -- mark the item done after reviewing the diff.
- [Snapshots](../concepts/snapshots.md) -- how baselines are stored and used.
