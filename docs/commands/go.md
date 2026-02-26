# plumb go

Start or reopen work on a queued item.

## Synopsis

```bash
plumb go <target>
```

## Behavior by Item State

### `todo`

1. verifies source exists, is a file, and is readable
2. captures baseline to `.plumb/sessions/<session_id>/snapshots/<item_id>.baseline`
3. marks item `in_progress`
4. opens editor

Success output (from state transition step):

```text
ok[PLB-OUT-SNP-001]: baseline captured and item started
```

### `in_progress`

- reopens editor only
- no baseline recapture
- no state change
- no structured success message

### `done`

- fails (`PLB-ITM-002`)

## Editor

- uses `EDITOR` env var when set/non-empty
- otherwise defaults to `vim`

## Important Note

If baseline capture and state update succeed but editor launch fails, the item remains `in_progress` with baseline already captured.

## Common Errors

- no active session: `PLB-SES-001`
- target not found: `PLB-ITM-001`
- already done: `PLB-ITM-002`
- source missing/dir/unreadable: `PLB-SNP-003/004/005`
- editor launch/exit failure: `PLB-EDT-001/002`
