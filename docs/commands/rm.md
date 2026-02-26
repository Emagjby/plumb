# plumb rm

Remove an item from the active session queue.

## Synopsis

```bash
plumb rm <target>
```

## Behavior

Resolves target by id or path, then:

- allows removal for `todo`
- allows removal for `done`
- rejects removal for `in_progress`

When removal succeeds, it also deletes baseline snapshot if present:

- `.plumb/sessions/<session_id>/snapshots/<item_id>.baseline`

Item ids are not renumbered.

## Output

```text
ok[PLB-OUT-ITM-005]: item removed from queue
```

Verbose includes `item_id` and `path`.

## Common Errors

- no active session: `PLB-SES-001`
- target not in queue: `PLB-ITM-001`
- target is in progress: `PLB-ITM-004`
