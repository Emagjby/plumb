# plumb add

Add files to the active session queue.

## Synopsis

```bash
plumb add <target>
plumb add -f <target>
```

## Behavior

All added items are created in `todo` state.

### File mode (`plumb add <target>`)

- normalizes target path relative to workspace root
- rejects directory targets
- duplicate path is an error
- file may be missing on disk at add time

Success output:

```text
ok[PLB-OUT-ITM-001]: item added to queue
```

### Folder mode (`plumb add -f <target>`)

- target must be a directory
- recursive walk
- skips directories named `.plumb`, `.git`, `target`, `node_modules`
- discovered files are sorted by normalized slash path before enqueue

Per-file outcomes:

- duplicate file: warning `PLB-OUT-ITM-003` (`action: skipped`)
- any per-file add failure: warning `PLB-OUT-ITM-004` (`action: skipped`)

Completion output:

```text
ok[PLB-OUT-ITM-002]: folder scan completed
```

Verbose includes `files_found` and `items_added`.

## Common Errors

- no active session: `PLB-SES-001`
- invalid path: `PLB-WSP-002`
- path escapes workspace: `PLB-WSP-003`
