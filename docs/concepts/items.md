# Items

An item is one queued file in a session.

## Stored Fields

Each item record in `items.scb` contains:

- `id` (`int`)
- `rel_path` (`string`)
- `state` (`string`: `todo`, `in_progress`, `done`)

## ID Assignment

New id is `max(existing_id) + 1`.

- ids start at `1` for an empty session
- removed id gaps are preserved
- ids are not compacted after `rm`

## Path Handling

- paths are stored normalized relative to workspace root
- separators are normalized to `/`
- duplicate detection compares normalized path forms

`plumb add` can queue a path that does not yet exist on disk.

## Folder Add Ordering

For `plumb add -f <dir>`:

- files are discovered recursively
- excluded dirs are skipped (`.plumb`, `.git`, `target`, `node_modules`)
- files are sorted lexicographically by normalized path before insertion

This makes bulk add deterministic.

## Related Commands

- [../commands/add.md](../commands/add.md)
- [../commands/rm.md](../commands/rm.md)
- [../commands/status.md](../commands/status.md)
