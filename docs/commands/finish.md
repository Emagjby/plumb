# plumb finish

Close the active session.

## Synopsis

```bash
plumb finish
```

## Behavior

1. fails if any item is still `in_progress`
2. writes `status = "finished"` into session metadata (`session.scb`)
3. removes `.plumb/active` (when it points to the finishing session)

Session files and snapshots remain on disk under `.plumb/sessions/<session_id>/`.

## Output

```text
ok[PLB-OUT-SES-003]: session finished
```

Verbose includes `session_id`.

## Common Errors

- no active session: `PLB-SES-001`
- at least one item still in progress: `PLB-SES-004`
