# Agent Contract v1

This document defines the machine interface for Imagenie v0.1. The canonical
JSON Schema is [`schemas/agent-events.schema.json`](../schemas/agent-events.schema.json).

## Stream rules

- `--json` emits one JSON object per line on stdout.
- Diagnostics and progress are written only to stderr.
- Every event includes `schema_version`, `request_id`, `event`, and `operator`.
- Per-input events include `input_index`, allowing agents to correlate duplicate
  paths and unordered parallel results.
- Every accepted processing invocation terminates with exactly one `summary`
  event.
- A `done` event means the output artifact was written successfully.
- A `skipped` event identifies an existing output intentionally left untouched.
- `--json` cannot be combined with binary stdout pipe mode. Agents should use a
  file output when they need structured events.

## Event lifecycle

```text
start -> done
start -> skipped
start -> error
... -> summary
```

Batch item events may be interleaved. `input_index`, not line ordering, is the
stable correlation key.

## Model identity

Processing events include the registry model name and the SHA-256 of its
download artifact. The hash pins the model payload used by the installation and
must change when the payload changes.

## Error recovery

Errors contain a stable `code`, human-readable `message`, and actionable
`recovery` instruction. Exit codes remain the process-level summary:

- `0`: all inputs succeeded or were explicitly skipped.
- `1`: partial or processing failure.
- `2`: invalid invocation.
- `3`: required model unavailable.
- `4`: all requested inputs were unreadable.

Agents should branch on the structured error code, not parse message text.

## Compatibility

`schema_version` follows semantic-version intent:

- New optional fields and new event types are backward-compatible minor
  additions.
- Removing, renaming, or changing the type or meaning of a field requires a new
  major schema version.
- Event consumers must ignore unknown optional fields and event types.
