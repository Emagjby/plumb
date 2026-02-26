# plumb diff

Compare current file contents to go-time baseline.

## Synopsis

```bash
plumb diff [target]
```

## Behavior

- with `target`: diffs that item (`id` or `path`)
- without `target`: diffs all items currently `in_progress`

No target + no `in_progress` items is a success no-op.

## Diff Model

`baseline bytes` vs `current file bytes`

- missing current file is treated as empty bytes
- if either side is non-UTF-8, output is:
  - `Binary files differ: <path>`
- unchanged files produce no output

## Output Format

`plumb diff` prints raw payload (unified diff or binary notice), not `PLB-OUT-*` envelopes.

## Common Errors

- no active session: `PLB-SES-001`
- target not in queue: `PLB-ITM-001`
- target is `todo` (no baseline yet): `PLB-SNP-001`
- baseline snapshot missing: `PLB-SNP-002`
