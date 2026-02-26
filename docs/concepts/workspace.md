# Workspace

Persistent state is stored under `.plumb/` at workspace root.

## Layout

```text
.plumb/
  active
  sessions/
    <session_id>/
      session.scb
      items.scb
      snapshots/
        <item_id>.baseline
```

## Files

### `.plumb/active`

Plain text active session id.

- created/updated by `start`
- removed by `finish`

### `session.scb`

SCB map with session metadata (`session_id`, `name`, `created_at`, optional `status`).

### `items.scb`

SCB list of item maps (`id`, `rel_path`, `state`).

### `snapshots/*.baseline`

Raw file bytes captured by `go`.

## Root Selection

Workspace root is nearest parent containing `.plumb/`.
If none exists, current directory is used.

## Writes

Core state writes use atomic file replacement for consistency.

## Notes

Add `.plumb/` to `.gitignore` in version-controlled repositories.
