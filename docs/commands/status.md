# plumb status

Print active session summary and queue listing.

## Synopsis

```bash
plumb status
```

## Behavior

Prints in this order:

1. structured header record
2. counts by state
3. `queue:` section with all items in stored order

Count lines:

- `<n> item(s) [TODO]`
- `<n> item(s) [IN_PROGRESS]`
- `<n> item(s) [DONE]`

Queue row format:

- `[<id>] <rel_path> - <state>`

## Output Header

```text
info[PLB-OUT-SES-002]: session status
```

Verbose header includes `session_id`.

## Common Errors

- no active session: `PLB-SES-001`
- store read/decode/schema failures: `PLB-STO-*`
