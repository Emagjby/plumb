# plumb next

Print the next `todo` item.

## Synopsis

```bash
plumb next
```

## Behavior

- finds the first `todo` item in stored list order
- does not mutate state

Success output summary is:

```text
info[PLB-OUT-ITM-007]: Next item: <path> (ID: <id>)
```

If no `todo` item exists, command fails (non-zero) with diagnostic `PLB-ITM-005`.

## Common Errors

- no active session: `PLB-SES-001`
- no `todo` items in queue: `PLB-ITM-005`
