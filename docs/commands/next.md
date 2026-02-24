# plumb next

Print the next todo item.

## Synopsis

```bash
plumb next
```

## Description

Prints the next `todo` item in the queue -- the one with the **lowest ID** among
all items still in `todo` state. This command is read-only; it does not change
any state.

Use `plumb next` to see what is coming up without committing to it. When you are
ready to start, use `plumb go` with the printed ID or path.

## Arguments

None.

## Options

None.

## Output

If there is a `todo` item:

```
Next: [3] src/middleware/cors.rs
```

If all items are `done` or `in_progress` (no `todo` items remain):

```
No todo items remaining.
```

## Examples

```bash
plumb next
# Next: [3] src/middleware/cors.rs
```

```bash
plumb next
# No todo items remaining.
```

## Notes

- **Read-only.** Does not change item state. The item stays `todo`.
- The "next" item is always the one with the lowest ID in `todo` state.
- Requires an active session.

## See also

- [plumb go](./go.md) -- start working on the next item.
- [plumb status](./status.md) -- see the full session summary.
- [States](../concepts/states.md)
