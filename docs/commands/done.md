# plumb done

Mark an `in_progress` item as `done`.

## Synopsis

```bash
plumb done <target>
```

`target` is required.

## Behavior

- resolves by item id or path
- succeeds only when target state is `in_progress`
- changes state `in_progress -> done`
- no default "current item" behavior exists in current implementation

Success output:

```text
ok[PLB-OUT-ITM-006]: item marked as done
```

Verbose includes `item_id` and `path`.

## Common Errors

- no active session: `PLB-SES-001`
- target not in queue: `PLB-ITM-001`
- target not `in_progress`: `PLB-ITM-003`
