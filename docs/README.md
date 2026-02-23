# Plumb Docs

This folder contains the specification and user manual for **Plumb** v0.1.

## Start here

- [Manual](./MANUAL.md) -- full reference for every command and concept.

## Quick example

- [Typical flow](./examples/typical-flow.md) -- a start-to-finish refactor session walkthrough.

## Core concepts

- [Sessions](./concepts/sessions.md) -- short-lived work containers.
- [Items](./concepts/items.md) -- files tracked inside a session.
- [States](./concepts/states.md) -- the three states an item moves through.
- [Snapshots](./concepts/snapshots.md) -- go-time baselines and diffing.
- [Workspace](./concepts/workspace.md) -- `.plumb/` storage layout and Strata SCB files.

## Command reference

| Command        | Page                           |
| -------------- | ------------------------------ |
| `plumb start`  | [start](./commands/start.md)   |
| `plumb add`    | [add](./commands/add.md)       |
| `plumb status` | [status](./commands/status.md) |
| `plumb go`     | [go](./commands/go.md)         |
| `plumb diff`   | [diff](./commands/diff.md)     |
| `plumb done`    | [done](./commands/done.md)       |
| `plumb restore` | [restore](./commands/restore.md) |
| `plumb next`    | [next](./commands/next.md)       |
| `plumb finish` | [finish](./commands/finish.md) |
