# Snapshots

A **snapshot** is a frozen copy of a file's contents captured at a specific
moment. Plumb uses snapshots to produce diffs that show exactly what changed
during a refactoring pass.

## Go-time baseline

When you run `plumb go <id|file>`, Plumb reads the file's current bytes from
disk and stores them as the **baseline snapshot**. This is the "before" image.
Plumb then opens the file in vim for editing. Everything you change after this
point is visible via `plumb diff`.

The baseline is captured **once**, at go-time. It is never updated. If you make
changes, then run `plumb diff`, the diff shows the full delta from the original
baseline to the file's current state on disk.

## Diff model

`plumb diff` performs a straightforward comparison:

```
baseline snapshot (captured at go-time)  vs.  current file on disk
```

The output is a unified diff. There is no three-way merge, no git index, and no
staging area.

## Storage format

Baseline snapshots are stored as **raw file bytes** -- the exact byte sequence
read from the file at go-time. They are NOT encoded as Strata SCB. This
preserves fidelity for any file type (text, binary, images, etc.).

Snapshot files live under:

```
.plumb/sessions/<session_id>/snapshots/<item_id>.baseline
```

An optional metadata sidecar may also be stored:

```
.plumb/sessions/<session_id>/snapshots/<item_id>.meta.scb
```

The `.meta.scb` file is a Strata SCB binary containing optional metadata about
the snapshot (e.g. capture timestamp, original file size). It is not required for
core functionality.

## Failure cases

| Scenario | Behavior |
|----------|----------|
| File does not exist at go-time | `plumb go` fails with an error. The item stays `todo`. |
| File is unreadable (permissions) | `plumb go` fails with an error. The item stays `todo`. |
| Baseline missing when running `plumb diff` | `plumb diff` fails with an error. This should not happen in normal use. |
| File deleted after go-time | `plumb diff` reports the file as deleted (all lines removed). |
| Baseline/current is non-UTF-8 bytes | `plumb diff` prints `Binary files differ: <path>`. |

## See also

- [States](./states.md) -- the `todo` -> `in_progress` transition triggers snapshot capture.
- [Items](./items.md) -- each item may have an associated baseline.
- [Workspace](./workspace.md) -- where snapshot files are stored on disk.
- [plumb go](../commands/go.md)
- [plumb diff](../commands/diff.md)
- [plumb restore](../commands/restore.md)
