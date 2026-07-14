# ADR 0001: Introduce a shared core operator facade

- Status: accepted
- Date: 2026-07-14
- Owners: Imagenie maintainers

## Context

The CLI and desktop app both constructed low-level ONNX model processors and
implemented workflow behavior around them. Background removal duplicated alpha
mask resizing and compositing in both adapters. Low-level processors also
returned only a `DynamicImage`, while agent callers need task metrics and
warnings through a consistent result contract.

The existing operators have two different shapes:

- Upscale, restore, and background removal use reusable configuration and can
  expose a simple `process(input)` entry point.
- Segment and erase have request-specific state: prompts, masks, dilation, and
  SAM embeddings that should be reused across prompts. Forcing them into a
  parameterless interface would either reload models or hide important state.

## Decision

`imagenie-core` owns the following public facade:

- `ImageOperator::process` and order-preserving `process_batch`, with one result
  per input so a single failure does not discard the rest of a batch.
- `ParameterizedImageOperator::process_with` for request-specific prompts and
  masks while the operator continues to own reusable sessions.
- `ProcessOutput`, containing the final image artifact, task metrics, and
  warnings.
- `ModelOperator<M>` for models whose postprocessed image is already final.
- `BackgroundRemover<M>` for the complete background-removal workflow,
  including alpha compositing.
- `SegmentOperator`, which caches the SAM embedding while the source path,
  length, and modification time remain unchanged.
- `EraseOperator`, which reuses the LaMa session and applies request-specific
  mask dilation in core.

The CLI uses these operators for all five v0.1 operations. The desktop app uses
them for its upscale, restore, and rembg features.
Output path selection, file collision policy, JSON events, Tauri commands, and
presentation remain adapter responsibilities.

## Alternatives considered

- **A single enum-based engine immediately:** rejected for this slice because it
  would combine model caching, request dispatch, output persistence, and five
  migrations into one high-risk change.
- **Keep only low-level `ModelProcessor`:** rejected because it allows adapters
  to duplicate product behavior and cannot carry metrics or warnings.
- **Configure segment prompts and erase masks at construction:** rejected
  because this makes model reuse inefficient for interactive agent workflows;
  parameterized calls keep this state request-scoped.

## Consequences

- CLI and desktop background removal now have one implementation.
- Basic model operators share one output and batch contract.
- Core has a small, testable pure function for alpha compositing.
- A future enum-based engine can wrap these traits without moving inference or
  workflow logic back into adapters.
- The raw model and processor modules remain public temporarily for migration
  compatibility.

## Validation

- Unit tests cover independent batch failures and alpha compositing behavior.
- Workspace tests and clippy cover CLI and desktop callers.
- The rembg output is compared byte-for-byte before and after migration on the
  same model and fixture.
- Core smoke examples run all five operations through the facade, including the
  composed segment-to-erase workflow.
