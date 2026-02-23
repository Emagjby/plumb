# plumb go

Start working on an item. Captures the baseline snapshot and opens the file in vim.

## Synopsis

```
plumb go <id|file>
```

## Description

Sets the specified `todo` item to `in_progress`, captures a **baseline
snapshot** of the file's current contents, and **opens the file in vim** for
editing. The baseline is a frozen copy of the file at this exact moment -- the
"before" image for `plumb diff`.

This is the critical transition in the Plumb workflow. After `plumb go`, you are
dropped straight into vim with the file open. Every change you make is visible
via `plumb diff` when you exit.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `id` or `file` | Yes | The item to start. Accepts either the integer ID or the file path (relative to workspace root). |

## Options

None.

## Baseline capture

When `plumb go` runs, it:

1. Reads the file's current bytes from disk.
2. Writes those bytes to `.plumb/sessions/<session_id>/snapshots/<item_id>.baseline`.
3. Transitions the item's state from `todo` to `in_progress`.
4. Opens the file in `vim`.

The baseline is **raw file bytes** -- the exact byte sequence on disk, with no
encoding or transformation.

An optional metadata sidecar (`<item_id>.meta.scb`, Strata SCB binary) may also
be written with capture metadata such as a timestamp.

## Examples

Start working on item 1:

```bash
plumb go 1
# Started: [1] src/auth/guard.rs (baseline captured)
# opens src/auth/guard.rs in vim
```

Start working on an item by path:

```bash
plumb go src/auth/guard.rs
# Started: [1] src/auth/guard.rs (baseline captured)
# opens src/auth/guard.rs in vim
```

## Notes

- **Single in-progress rule.** Only one item may be `in_progress` at a time. If
  another item is already `in_progress`, `plumb go` fails with an error. Run
  `plumb done` on the current item first.
- **File must exist.** If the file does not exist on disk, `plumb go` fails and
  the item stays `todo`.
- **File must be readable.** If the file cannot be read (e.g. permission denied),
  `plumb go` fails and the item stays `todo`.
- **Item must be `todo`.** Running `plumb go` on a `done` item is an error.
- **Opens vim.** After capturing the baseline, Plumb opens the file in `vim`.
  Plumb waits for vim to exit before returning control to the terminal.
- Requires an active session.

## See also

- [plumb diff](./diff.md) -- see what changed since the baseline.
- [plumb restore](./restore.md) -- revert the file to the baseline.
- [plumb done](./done.md) -- mark the item as done.
- [Snapshots](../concepts/snapshots.md) -- how baselines work.
- [States](../concepts/states.md) -- the `todo` -> `in_progress` transition.
