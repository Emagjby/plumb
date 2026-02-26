# plumb start

Begin a new session.

## Synopsis

```bash
plumb start [name]
```

## Behavior

1. Resolves workspace root (nearest parent with `.plumb/`, else current dir).
2. Ensures `.plumb/` exists.
3. Fails if `.plumb/active` already points to a valid active session id.
4. Creates:
   - `.plumb/sessions/<session_id>/session.scb`
   - `.plumb/sessions/<session_id>/items.scb` (empty list)
   - `.plumb/sessions/<session_id>/snapshots/`
5. Writes `.plumb/active` with session id.

`name` is optional metadata only.

## Output

Default:

```text
ok[PLB-OUT-SES-001]: session started
```

Verbose adds `session_id` and optional `name` context.

## Common Errors

- existing active session: `PLB-SES-002`
- invalid/corrupt active pointer: `PLB-SES-003`
