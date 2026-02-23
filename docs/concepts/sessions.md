# Sessions

A **session** is a short-lived work container for a single refactoring effort.
You start one, enqueue the files you plan to touch, work through them one at a
time, and close the session when you are finished.

## Lifecycle

1. **Created** -- `plumb start` creates a session and marks it active.
2. **Active** -- You add items, pick them off, refactor, and mark them done.
3. **Finished** -- `plumb finish` closes the session and clears the active pointer.

## Active session rule

There is **exactly one active session at a time**. Every command that operates
on a session (`add`, `go`, `status`, etc.) targets the active session
implicitly. If no session is active the command fails with an error.

Starting a new session while another is active is an error. You must `finish`
the current session first.

## Session identity

Each session receives a unique identifier (generated at creation time). The
active session ID is recorded in `.plumb/active` so that Plumb can find it on
subsequent invocations.

## Optional session name

`plumb start` accepts an optional human-readable name (e.g.
`"refactor auth guards"`). The name is stored in the session's metadata but is
not used as an identifier.

## Storage

Session metadata is persisted in a Strata SCB file at:

```
.plumb/sessions/<session_id>/session.scb
```

See [Workspace](./workspace.md) for the full layout.

## See also

- [Items](./items.md) -- the files tracked inside a session.
- [States](./states.md) -- how items move through the session.
- [plumb start](../commands/start.md)
- [plumb finish](../commands/finish.md)
