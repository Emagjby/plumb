# plumb restore

Restore a file to its baseline snapshot.

## Synopsis

```bash
plumb restore <target>
```

`target` is required.

## Behavior

1. resolves target by id or path
2. rejects `todo` items (no baseline exists)
3. verifies destination file exists, is not a directory, and is writable
4. loads baseline snapshot bytes
5. asks for confirmation
6. if confirmed, atomically overwrites destination with baseline bytes

Restore does not change item state.

## Prompt and Confirmation

Prompt output record:

```text
prompt[PLB-OUT-SNP-002]: restore file to baseline snapshot (note: all changes since go-time will be lost)
```

Then:

```text
Are you sure? [y/N]
```

Accepted confirmation: `y` or `yes` (case-insensitive).
Any other input cancels.

Cancel output:

```text
info[PLB-OUT-SNP-003]: Restore cancelled.
```

Confirm output:

```text
ok[PLB-OUT-SNP-004]: file restored to baseline snapshot
```

## Common Errors

- no active session: `PLB-SES-001`
- target not in queue: `PLB-ITM-001`
- no baseline yet (`todo`): `PLB-SNP-001`
- baseline missing on disk: `PLB-SNP-002`
- destination missing/dir/not writable: `PLB-SNP-006/007/008`
