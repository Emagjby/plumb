# Workspace

Plumb stores all persistent state in a `.plumb/` directory at the workspace
root. This directory is created automatically by `plumb start` if it does not
already exist.

## Directory layout

```
.plumb/
  active                          # plain text file: the active session ID (or empty)
  sessions/
    <session_id>/
      session.scb                 # session metadata (Strata SCB binary)
      items.scb                   # all items in this session (Strata SCB binary)
      snapshots/
        <item_id>.baseline        # raw file bytes captured at go-time
        <item_id>.meta.scb        # optional snapshot metadata (Strata SCB binary)
```

## File descriptions

### `active`

A plain text file containing the active session's ID (or empty / absent if no
session is active). This is the one file in `.plumb/` that is **not** Strata SCB
-- it is kept as plain text for simplicity so that the active pointer can be read
with trivial I/O.

### `session.scb`

A Strata SCB binary file storing session-level metadata: the session ID, an
optional human-readable name, creation timestamp, and status (active or
finished).

### `items.scb`

A Strata SCB binary file storing the list of items (files) in the session. Each
item record contains its integer ID, relative path, current state, and an
optional reference to its baseline snapshot.

### `<item_id>.baseline`

The baseline snapshot: raw bytes of the file as it existed at the moment
`plumb go` was run. Stored without any encoding or transformation.

### `<item_id>.meta.scb`

Optional. A Strata SCB binary file with metadata about the baseline snapshot
(e.g. capture timestamp, byte length). Not required for core functionality.

## Strata SCB

All `*.scb` files are **Strata Core Binary** -- a deterministic, self-delimiting,
language-neutral binary format. Key properties:

- Every value has exactly one canonical byte representation.
- Files are not human-readable; use Strata tooling to inspect.
- The format is hash-stable: identical logical values always produce identical bytes.

See the [Strata documentation](https://strata.emagjby.com/docs) for the full
specification.

## Paths

All file paths stored by Plumb (in `items.scb`, etc.) are **relative to the
workspace root** -- the directory containing `.plumb/`. This keeps sessions
portable within a machine if the workspace is moved.

## Ignoring `.plumb/`

The `.plumb/` directory is internal state. It should be added to `.gitignore`
(or equivalent) if the workspace is under version control.

## See also

- [Sessions](./sessions.md) -- what gets stored in `session.scb`.
- [Items](./items.md) -- what gets stored in `items.scb`.
- [Snapshots](./snapshots.md) -- what baseline files contain.
- [plumb start](../commands/start.md) -- creates the `.plumb/` directory.
