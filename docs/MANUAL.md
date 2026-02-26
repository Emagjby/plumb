# plumb(1) Manual

## Overview

`plumb` is a local CLI that runs a refactor session as a queue of files.

Core loop:

1. `plumb start [name]`
2. `plumb add <path>` or `plumb add -f <dir>`
3. `plumb go <id|path>`
4. `plumb diff [id|path]`
5. `plumb done <id|path>` or `plumb restore <id|path>`
6. `plumb finish`

## Global Flag

```bash
plumb --verbose <command>
plumb -v <command>
```

`-v` / `--verbose` is global and applies to every command.

## Workspace Root Resolution

For each command, `plumb` walks upward from current directory to find the nearest `.plumb/`.

- if found: that directory is the workspace root
- if not found: current directory is treated as root

## Commands

| Command | Signature | What it does |
| --- | --- | --- |
| `start` | `plumb start [name]` | Creates session metadata/items/snapshots and writes `.plumb/active`. |
| `add` | `plumb add <target>` | Adds one queue item as `todo`. |
| `add -f` | `plumb add -f <dir>` | Recursively adds files, skipping `.plumb`, `.git`, `target`, `node_modules`. |
| `rm` | `plumb rm <target>` | Removes `todo`/`done` item and deletes its baseline snapshot if present. |
| `status` | `plumb status` | Prints session header, state counts, and full queue listing. |
| `go` | `plumb go <target>` | For `todo`: capture baseline, mark `in_progress`, open editor. For `in_progress`: reopen editor only. |
| `diff` | `plumb diff [target]` | Diffs one target, or all `in_progress` items when omitted. |
| `done` | `plumb done <target>` | Marks an `in_progress` item as `done`. |
| `next` | `plumb next` | Prints first `todo` item in stored order. |
| `restore` | `plumb restore <target>` | Confirms then overwrites file with baseline bytes. |
| `finish` | `plumb finish` | Refuses if any item is `in_progress`; otherwise marks session finished and removes `.plumb/active`. |

## Session and Item Rules

- session id format is 8 lowercase hex characters
- item states are `todo`, `in_progress`, `done`
- current implementation allows multiple `in_progress` items
- item ids are `max(existing_id) + 1` (gaps are not compacted)

## Snapshot Rules

- baseline captured only when `go` starts a `todo` item
- snapshot path: `.plumb/sessions/<session_id>/snapshots/<item_id>.baseline`
- snapshot bytes are raw file bytes
- reopening an `in_progress` item with `go` does not recapture baseline

## Diff Rules

- compares baseline bytes vs current file bytes
- missing current file is treated as empty content
- non-UTF-8 side prints `Binary files differ: <path>`
- unchanged content prints nothing
- `plumb diff` intentionally prints raw diff payload (no output envelope)

## Restore Rules

`restore` requires target argument and prompts for confirmation.

- accepts `y` / `yes` (case-insensitive) as confirmation
- anything else cancels restore
- cancelled restore exits success and leaves file unchanged
- restore does not change item state

## Finish Rules

`finish` performs:

1. rejects when any item is `in_progress`
2. updates `session.scb` with `status = "finished"`
3. removes `.plumb/active` when it points to that session

`todo` items may remain when finishing.

## Output and Error Standards

- fatal diagnostics use `error[PLB-...]` records with actionable hints
- non-fatal command output uses `ok/info/warn/prompt[PLB-OUT-...]` records

Default mode is collapsed one-line records.
Verbose mode includes command/context fields.
