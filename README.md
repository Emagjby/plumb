# plumb

A CLI for running refactor sessions as a disciplined queue of files.

```
plumb start "refactor auth"
plumb add src/auth/guard.rs
plumb add -f src/middleware/
plumb go 1
plumb diff
plumb done
plumb finish
```

## What it does

Plumb tracks files through three states: `todo`, `in_progress`, `done`.
When you start working on a file (`plumb go`), a baseline snapshot is captured
and the file opens in vim. When you're done, `plumb diff` shows exactly what
changed against that baseline. One file at a time, one session at a time.

## Commands

```
start [name]       begin a new session
add <file>         enqueue a file as todo
add -f <folder>    enqueue all files in a folder (recursive)
rm <id|file>       removes a file from the queue
status             show session progress
go <id|file>       set item in_progress, capture baseline, open vim
diff [id|file]     diff current file against baseline
done [id|file]     mark in_progress item as done
restore [id|file]  revert file to baseline (confirms before overwriting)
next               print next todo item
finish             close the session
```

## Storage

State is stored locally under `.plumb/` using Strata SCB binary format.
Baseline snapshots are raw file bytes. No external dependencies at runtime.

## Status

v0.1 -- local only, no git integration, no AI integration.

## Docs

Full manual and command reference: [docs/](./docs/README.md)
