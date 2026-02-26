# Typical Flow

Example session using real current command behavior.

## 1. Start

```bash
plumb start "refactor auth"
# ok[PLB-OUT-SES-001]: session started
```

## 2. Add files

```bash
plumb add src/auth/guard.rs
# ok[PLB-OUT-ITM-001]: item added to queue

plumb add src/auth/session.rs
# ok[PLB-OUT-ITM-001]: item added to queue
```

Optional folder add:

```bash
plumb add -f src/middleware
# ok[PLB-OUT-ITM-002]: folder scan completed
```

## 3. Check status

```bash
plumb status
# info[PLB-OUT-SES-002]: session status
#   3 item(s) [TODO]
#   0 item(s) [IN_PROGRESS]
#   0 item(s) [DONE]
#
# queue:
#   [1] src/auth/guard.rs - todo
#   [2] src/auth/session.rs - todo
#   [3] src/middleware/cors.rs - todo
```

## 4. Start work on first item

```bash
EDITOR=true plumb go 1
# ok[PLB-OUT-SNP-001]: baseline captured and item started
```

This writes baseline bytes to:

```text
.plumb/sessions/<session_id>/snapshots/1.baseline
```

## 5. Review changes

```bash
plumb diff 1
# --- baseline: src/auth/guard.rs
# +++ current:  src/auth/guard.rs
# @@ ...
```

## 6. Mark done

```bash
plumb done 1
# ok[PLB-OUT-ITM-006]: item marked as done
```

## 7. See next item

```bash
plumb next
# info[PLB-OUT-ITM-007]: Next item: src/auth/session.rs (ID: 2)
```

## 8. Optional restore flow

```bash
plumb restore 2
# prompt[PLB-OUT-SNP-002]: restore file to baseline snapshot (note: all changes since go-time will be lost)
# Are you sure? [y/N] y
# ok[PLB-OUT-SNP-004]: file restored to baseline snapshot
```

## 9. Finish session

`finish` requires no `in_progress` items.

```bash
plumb finish
# ok[PLB-OUT-SES-003]: session finished
```

After finish, `.plumb/active` is removed and session metadata is marked `finished`.
