# States

Each item has one state: `todo`, `in_progress`, or `done`.

## Meanings

- `todo`: queued, not started
- `in_progress`: baseline captured and item opened for work
- `done`: marked complete

## Transitions

Implemented transitions:

- `todo -> in_progress` via `plumb go`
- `in_progress -> done` via `plumb done`

Other behaviors:

- `restore` keeps current state unchanged
- `rm` removes item entirely (`todo`/`done` only)

## Important Reality

Current implementation does not enforce a single `in_progress` item.
Multiple items can be `in_progress` at once.

## Related Commands

- [../commands/go.md](../commands/go.md)
- [../commands/done.md](../commands/done.md)
- [../commands/status.md](../commands/status.md)
