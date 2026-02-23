# plumb done

Mark the in-progress item as done.

## Synopsis

```
plumb done [id|file]
```

## Description

Transitions an `in_progress` item to `done`. By default (no arguments), it
operates on the **currently in-progress item**. You may also specify an item
explicitly by ID or file path.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `id` or `file` | No | The item to mark done. Defaults to the current `in_progress` item. Accepts an integer ID or a file path relative to the workspace root. |

## Options

None.

## Examples

Mark the current in-progress item as done (most common):

```bash
plumb done
# Done: [1] src/auth/guard.rs
```

Mark a specific item by ID:

```bash
plumb done 1
# Done: [1] src/auth/guard.rs
```

Mark a specific item by path:

```bash
plumb done src/auth/guard.rs
# Done: [1] src/auth/guard.rs
```

## Notes

- **Default target.** When called with no arguments, `plumb done` marks the
  single `in_progress` item. If no item is `in_progress`, the command fails.
- **Item must be `in_progress`.** You cannot mark a `todo` or `done` item as
  done.
- If you specify an item that is not `in_progress`, the command fails with an
  error.
- Requires an active session.
- After marking an item done, there is no `in_progress` item until you run
  `plumb go` again.

## See also

- [plumb go](./go.md) -- start the next item.
- [plumb next](./next.md) -- see what is next in the queue.
- [plumb diff](./diff.md) -- review changes before marking done.
- [plumb restore](./restore.md) -- revert the file instead of marking done.
- [States](../concepts/states.md) -- the `in_progress` -> `done` transition.
