# Sessions

A session is the container for one refactor run.

## Lifecycle

1. `start` creates a new session and marks it active.
2. commands operate against the active session.
3. `finish` marks session finished and removes active pointer.

## Active Session Pointer

Active session is tracked by `.plumb/active`.

- when active: file exists and contains session id
- when no active session: file is absent (or treated as inactive if invalid/empty)

`start` refuses when `.plumb/active` already contains a valid session id.

## Session Metadata

`session.scb` stores:

- `session_id` (bytes)
- `name` (string, optional/empty)
- `created_at` (16-byte little-endian i128 nanoseconds)
- `status = "finished"` is added on `finish`

## Related Commands

- [../commands/start.md](../commands/start.md)
- [../commands/finish.md](../commands/finish.md)
