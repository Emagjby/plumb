# Typical flow

A complete walkthrough of a Plumb refactoring session, from start to finish.

## Scenario

You are refactoring the authentication module. There are two files to touch
individually and an entire middleware directory.

## 1. Start a session

```bash
plumb start "refactor auth module"
# Session a1b2c3d4 started.
```

This creates `.plumb/` (if it does not exist) and initialises a new session with
Strata SCB files for metadata and items.

## 2. Add files

Add individual files:

```bash
plumb add src/auth/guard.rs
# Added: [1] src/auth/guard.rs

plumb add src/auth/session.rs
# Added: [2] src/auth/session.rs
```

Add an entire folder:

```bash
plumb add -f src/middleware/
# Added: [3] src/middleware/cors.rs
# Added: [4] src/middleware/logging.rs
# Added: [5] src/middleware/rate_limit.rs
```

The folder contents are added in **lexicographic order** by relative path. IDs
continue from where the last add left off.

## 3. Check status

```bash
plumb status
# Session: refactor auth module
#
#   todo:        5
#   in_progress: 0
#   done:        0
#
# No item in progress.
```

Five items queued, nothing started yet.

## 4. Start the first item

```bash
plumb go 1
# Started: [1] src/auth/guard.rs (baseline captured)
```

**This is the key moment.** Plumb reads `src/auth/guard.rs` from disk and saves
a byte-for-byte copy as the baseline snapshot under
`.plumb/sessions/a1b2c3d4/snapshots/1.baseline`. Then it opens the file in
**vim**. Everything you change from this point forward is trackable.

## 5. Refactor the file

You are now inside vim with `src/auth/guard.rs` open. Make your changes, then
save and quit (`:wq`).

## 6. Diff to see what changed

```bash
plumb diff
# --- baseline: src/auth/guard.rs
# +++ current:  src/auth/guard.rs
# @@ -10,7 +10,7 @@
#  fn check_auth(req: &Request) -> bool {
# -    req.headers.contains("Authorization")
# +    req.headers.get("Authorization").is_some_and(|v| !v.is_empty())
#  }
```

The diff compares the **baseline snapshot** (from step 4) against the **current
file on disk**. This proves exactly what your refactoring changed.

## 7. Mark it done

```bash
plumb done
# Done: [1] src/auth/guard.rs
```

The item moves from `in_progress` to `done`. There is now no in-progress item.

## 8. See what is next

```bash
plumb next
# Next: [2] src/auth/session.rs
```

`plumb next` shows the lowest-ID `todo` item without changing any state.

## 9. Continue the loop

Repeat the go / edit / diff / done cycle for each remaining item:

```bash
plumb go 2
# Started: [2] src/auth/session.rs (baseline captured)

# ... edit src/auth/session.rs ...

plumb diff
# (shows changes)

plumb done
# Done: [2] src/auth/session.rs

plumb go 3
# Started: [3] src/middleware/cors.rs (baseline captured)

# ... edit ...

plumb diff
plumb done

# ... repeat for items 4 and 5 ...
```

## 10. Check final status

```bash
plumb status
# Session: refactor auth module
#
#   todo:        0
#   in_progress: 0
#   done:        5
#
# No item in progress.
```

All items are done.

## 11. Finish the session

```bash
plumb finish
# Session a1b2c3d4 finished.
```

The session is closed and the active pointer is cleared. Session data (SCB
files, snapshots) remains on disk under `.plumb/sessions/a1b2c3d4/`.

## Key ideas

- **`go` captures the baseline.** The snapshot is taken at the moment you start
  working, not when you added the file. This means the baseline reflects the
  actual state of the file just before you touched it.
- **`diff` proves the change.** By comparing current disk contents against the
  go-time baseline, you get a clean view of exactly what your refactoring did.
- **`done` closes the loop.** One file at a time, start to finish. The single
  in-progress rule enforces discipline.
- **Folder add is bulk queueing.** `plumb add -f` fills the queue quickly;
  you still work through items one at a time.

## See also

- [Manual](../MANUAL.md) -- full reference.
- [Sessions](../concepts/sessions.md)
- [Snapshots](../concepts/snapshots.md)
- [States](../concepts/states.md)
