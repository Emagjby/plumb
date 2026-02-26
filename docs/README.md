# Plumb Docs

Documentation for the current `plumb` implementation.

## Start Here

- [Manual](./MANUAL.md) - complete behavior and command reference
- [Typical flow](./examples/typical-flow.md) - end-to-end walkthrough

## Command Reference

- [start](./commands/start.md)
- [add](./commands/add.md)
- [rm](./commands/rm.md)
- [status](./commands/status.md)
- [go](./commands/go.md)
- [diff](./commands/diff.md)
- [done](./commands/done.md)
- [next](./commands/next.md)
- [restore](./commands/restore.md)
- [finish](./commands/finish.md)

## Concepts

- [workspace](./concepts/workspace.md)
- [sessions](./concepts/sessions.md)
- [items](./concepts/items.md)
- [states](./concepts/states.md)
- [snapshots](./concepts/snapshots.md)

## Output and Errors

- Fatal diagnostics are emitted as `error[PLB-...]`.
- Non-fatal command messages are emitted as `ok/info/warn/prompt[PLB-OUT-...]`.
- Command-specific examples are documented in each page under `docs/commands/`.

## Verbosity

All commands support global `-v` / `--verbose`.

- default: collapsed one-line records
- verbose: expanded command/context fields
