# Snapshots

A baseline snapshot is the file bytes captured when work starts.

## Capture Rules

Snapshot is captured only when `plumb go <target>` starts a `todo` item.

- snapshot file: `.plumb/sessions/<session_id>/snapshots/<item_id>.baseline`
- stored as raw bytes (not SCB)

Running `go` again on an already `in_progress` item reopens editor and does not recapture baseline.

## Use Sites

- `plumb diff` compares baseline bytes vs current bytes
- `plumb restore` overwrites current file from baseline bytes

## Missing File Semantics

- diff: missing current file is treated as empty content
- restore: missing destination file is an error

## Cleanup

`plumb rm` deletes `<item_id>.baseline` when the item is removed.

## Related Commands

- [../commands/go.md](../commands/go.md)
- [../commands/diff.md](../commands/diff.md)
- [../commands/restore.md](../commands/restore.md)
