# Plumb Error Standard (Draft)

## 1. Purpose

Define one consistent error description standard for all Plumb Rust modules and commands, so failures read like they come from one ecosystem.

This spec covers the current workspace:

- `plumb/src/main.rs`
- `plumb/src/error.rs`
- `plumb/src/workspace.rs`
- `plumb/src/fs.rs`
- `plumb/src/helpers.rs`
- `plumb/src/store/items.rs`
- `plumb/src/store/session.rs`
- `plumb/src/commands/*.rs`
- `plumb/tests/integration_*.rs`

## 2. Design Principles

- Use stable error codes for every user-visible failure.
- Keep first-line summaries short and actionable.
- Distinguish fatal errors from warnings and prompts.
- Preserve command guidance (`plumb start`, `plumb go`, etc.) in hints.
- Never surface raw `UnknownError` wording to users.
- Use consistent vocabulary:
- `session`: active refactor session.
- `item`: queue entry (`id`, `rel_path`, `state`).
- `target`: user-provided ID/path argument.
- `path`: resolved workspace-relative path.

## 3. Output Format

### 3.1 Fatal error (command exits non-zero)

```
error[<CODE>]: <summary>
  command: plumb <command>
  <context_key>: <context_value>
  hint: <how to recover>
  cause: <optional low-level cause>
```

Rules:

- First line is mandatory.
- `code` and `summary` are mandatory.
- `hint` is required for user-actionable failures.
- `cause` is optional and should be plain text (no debug dumps).
- Context keys must use `snake_case`.

### 3.2 Warning (command continues and exits 0)

```
warn[<CODE>]: <summary>
  <context_key>: <context_value>
  action: skipped
```

Use only for recoverable per-item failures in batch flows (for example `add -f` duplicates).

### 3.3 Prompt (interactive, not an error)

Prompts must not use `error[...]`/`warn[...]`. Keep current style with explicit risk and default-safe answer.

## 4. Taxonomy

Code format:

```
PLB-<DOMAIN>-<NNN>
```

Domains:

- `SES`: session lifecycle / active pointer
- `ITM`: queue item resolution and state rules
- `SNP`: baseline snapshot / diff / restore semantics
- `WSP`: workspace and path normalization
- `STO`: persisted state (`*.scb`, active file) read/write/shape
- `IO`: filesystem/terminal runtime I/O
- `EDT`: external editor execution
- `INT`: internal invariant violations

## 5. Canonical Code Registry

| Code | Summary | Typical hint |
| --- | --- | --- |
| `PLB-SES-001` | no active session found | run `plumb start [name]` |
| `PLB-SES-002` | active session already exists | run `plumb finish` first |
| `PLB-SES-003` | active session pointer is invalid | inspect `.plumb/active` and session dirs |
| `PLB-SES-004` | cannot finish while items are in progress | run `plumb done <id|path>` first |
| `PLB-ITM-001` | target item not found in queue | run `plumb status` to list IDs and paths |
| `PLB-ITM-002` | item is already done | choose a `todo` or `in_progress` item |
| `PLB-ITM-003` | item is not in_progress | run `plumb go <id|path>` first |
| `PLB-ITM-004` | cannot remove item in_progress | run `plumb done` or `plumb restore` first |
| `PLB-ITM-005` | no todo items in queue | run `plumb add <path>` |
| `PLB-ITM-006` | item already in queue | no action needed (skip) |
| `PLB-ITM-007` | target is a directory but file was expected | use `plumb add -f <folder>` |
| `PLB-SNP-001` | baseline snapshot required | run `plumb go <id|path>` first |
| `PLB-SNP-002` | baseline snapshot missing on disk | retry `plumb go` or inspect session snapshots |
| `PLB-SNP-003` | baseline source file does not exist | recreate file or choose another item |
| `PLB-SNP-004` | baseline source is a directory | provide a file item target |
| `PLB-SNP-005` | failed to read baseline source file | check permissions and file health |
| `PLB-SNP-006` | restore destination file does not exist | recreate file or skip restore |
| `PLB-SNP-007` | restore destination is a directory | provide a file item target |
| `PLB-SNP-008` | restore destination is not writable | fix permissions and retry |
| `PLB-WSP-001` | failed to resolve workspace root | run command inside a valid workspace |
| `PLB-WSP-002` | invalid input path | pass a valid path |
| `PLB-WSP-003` | path escapes workspace root | use a path inside the workspace |
| `PLB-WSP-004` | workspace layout is invalid | repair `.plumb` structure |
| `PLB-STO-001` | failed to read state file | verify file exists and is readable |
| `PLB-STO-002` | failed to decode state file | state file is corrupted or invalid |
| `PLB-STO-003` | state file has invalid schema | regenerate state or repair file |
| `PLB-STO-004` | failed to encode state file | retry and report if persistent |
| `PLB-STO-005` | failed to write state file | check permissions/disk space |
| `PLB-IO-001` | filesystem operation failed | inspect `cause` |
| `PLB-IO-002` | permission denied | fix permissions and retry |
| `PLB-IO-003` | terminal I/O failed | retry command in interactive shell |
| `PLB-EDT-001` | failed to launch editor | set valid `EDITOR` |
| `PLB-EDT-002` | editor exited with failure | inspect editor command/status |
| `PLB-INT-001` | internal invariant violation | re-run with details and report bug |

## 6. Message Conventions

- Summary text:
- lowercase sentence case.
- no trailing punctuation.
- <= 80 chars.
- no duplicate prefixes like `go error: ...`.
- Prefer specific nouns over generic:
- use `item` for queue objects.
- use `session` for active pointer/session lifecycle.
- use `baseline snapshot` for go-time file captures.
- Context order (when present):
- `command`
- `workspace`
- `session_id`
- `item_id`
- `target`
- `path`
- `expected_state`
- `actual_state`
- `hint`
- `cause`
- Do not include Rust debug formatting (`{:?}`) in user-facing summaries.
- Keep recoverable hints imperative and concrete.

## 7. Exit Behavior

- Fatal diagnostics (`error[...]`): non-zero exit.
- Warnings (`warn[...]` only): zero exit.
- Pure prompts/info: zero exit unless a fatal error follows.
- Clap parse/usage failures remain clap-controlled exits.

## 8. Coverage Map By Module

| Module | Primary codes |
| --- | --- |
| `commands/start.rs` | `PLB-SES-002`, `PLB-SES-003`, `PLB-WSP-001`, `PLB-STO-005` |
| `commands/add.rs` | `PLB-SES-001`, `PLB-WSP-002`, `PLB-WSP-003`, `PLB-ITM-006`, `PLB-ITM-007`, `PLB-IO-001` |
| `commands/rm.rs` | `PLB-SES-001`, `PLB-ITM-001`, `PLB-ITM-004`, `PLB-IO-001` |
| `commands/status.rs` | `PLB-SES-001`, `PLB-STO-001`, `PLB-STO-002`, `PLB-STO-003` |
| `commands/go.rs` | `PLB-SES-001`, `PLB-ITM-001`, `PLB-ITM-002`, `PLB-SNP-003`, `PLB-SNP-004`, `PLB-SNP-005`, `PLB-EDT-001`, `PLB-EDT-002` |
| `commands/diff.rs` | `PLB-SES-001`, `PLB-ITM-001`, `PLB-SNP-001`, `PLB-SNP-002`, `PLB-IO-001` |
| `commands/done.rs` | `PLB-SES-001`, `PLB-ITM-001`, `PLB-ITM-003`, `PLB-STO-005` |
| `commands/next.rs` | `PLB-SES-001`, `PLB-ITM-005` |
| `commands/restore.rs` | `PLB-SES-001`, `PLB-ITM-001`, `PLB-SNP-001`, `PLB-SNP-002`, `PLB-SNP-006`, `PLB-SNP-007`, `PLB-SNP-008`, `PLB-IO-003` |
| `commands/finish.rs` | `PLB-SES-001`, `PLB-SES-004`, `PLB-STO-001`, `PLB-STO-002`, `PLB-STO-005` |
| `workspace.rs` | `PLB-SES-002`, `PLB-SES-003`, `PLB-WSP-004`, `PLB-STO-004`, `PLB-STO-005` |
| `store/items.rs` | `PLB-SES-001`, `PLB-WSP-001`, `PLB-STO-001`, `PLB-STO-002`, `PLB-STO-003`, `PLB-STO-004`, `PLB-STO-005` |
| `store/session.rs` | `PLB-STO-001`, `PLB-STO-002`, `PLB-STO-003`, `PLB-STO-004`, `PLB-STO-005` |
| `helpers.rs` | `PLB-ITM-001`, `PLB-SNP-002`, `PLB-WSP-002`, `PLB-WSP-003` |
| `fs.rs` | `PLB-WSP-002`, `PLB-WSP-003`, `PLB-IO-001`, `PLB-IO-002` |
| `error.rs` / `main.rs` | renderer only; should forward canonical diagnostics unchanged |

## 9. Examples

### 9.1 No active session

```
error[PLB-SES-001]: no active session found
  command: plumb add
  hint: run `plumb start [name]`
```

### 9.2 Target not in queue

```
error[PLB-ITM-001]: target item not found in queue
  command: plumb done
  target: 99
  hint: run `plumb status` to list valid item IDs and paths
```

### 9.3 Baseline missing (`diff`/`restore` before `go`)

```
error[PLB-SNP-001]: baseline snapshot required
  command: plumb diff
  target: src/a.rs
  hint: run `plumb go <id|path>` first
```

### 9.4 Cannot finish with in-progress item

```
error[PLB-SES-004]: cannot finish while items are in progress
  command: plumb finish
  hint: run `plumb done <id|path>` for all in-progress items
```

### 9.5 Directory passed to file add

```
error[PLB-ITM-007]: target is a directory but file was expected
  command: plumb add
  path: src
  hint: use `plumb add -f src`
```

### 9.6 Duplicate during folder add (recoverable)

```
warn[PLB-ITM-006]: item already in queue
  command: plumb add -f
  path: src/a.rs
  action: skipped
```

### 9.7 Editor launch failure

```
error[PLB-EDT-001]: failed to launch editor
  command: plumb go
  path: src/a.rs
  hint: set `EDITOR` to a valid executable
  cause: No such file or directory (os error 2)
```

### 9.8 Corrupted persisted state

```
error[PLB-STO-002]: failed to decode state file
  command: plumb status
  path: .plumb/sessions/deadbeef/items.scb
  hint: state file is corrupted; regenerate or repair it
```

## 10. Implementation Notes

- Replace current free-form `String` wrappers with a shared diagnostic type (`code`, `summary`, `context`, `hint`, `cause`).
- Keep integration tests focused on code + summary/hint semantics, not incidental phrasing.
- Migrate command-by-command, starting with repeated session/item/snapshot failures currently asserted in integration tests.
