# v0.1 Product Specification

## Product promise

Imagenie gives an agent a local, private, and predictable way to perform common
image transformations. A successful call produces both an image artifact and
enough structured evidence for the agent to decide what to do next.

## Primary users

- Coding and desktop agents operating on local files.
- Automated media workflows that cannot upload source images to a cloud API.
- Humans using the same CLI for inspection and recovery.

## v0.1 scope

The release contains five operators:

- `upscale`: 2x local super-resolution.
- `rembg`: fast and high-quality background removal.
- `restore`: face restoration.
- `segment`: point- or box-prompted object segmentation.
- `erase`: mask-guided inpainting, including composition with `segment`.

Model management, image inspection, capability discovery, batch execution, and
machine-readable events are part of the product surface.

## Non-goals

- Adding more image operators before v0.1 is reliable.
- Training or converting original models.
- Making MCP the inference implementation; adapters remain thin wrappers.
- Optimizing the desktop UI before the shared core API is stable.

## Success measures

- **Agent task success:** at least 95 of 100 representative tasks complete in a
  single invocation and at least 90 meet the operator quality threshold.
- **Interface reliability:** at least 99% of valid invocations return
  schema-valid output; malformed inputs never panic.
- **Quality:** model updates do not regress more than 2% on an operator's
  primary metric unless a documented trade-off is approved.
- **Performance:** cold start, warm execution, peak RSS, and batch throughput
  are recorded separately on each reference machine.
- **Distribution:** supported binaries and model artifacts are reproducible,
  checksummed, licensed, and documented.

## Reference environments

The benchmark report must identify CPU, memory, operating system, execution
provider, Imagenie commit, model hash, input dimensions, and job count. Initial
baselines should include one Apple Silicon macOS machine and one x86-64 Linux
machine. Windows is a release compatibility target even when it is not a
performance reference machine.

## Definition of done

A user-visible change is complete only when:

1. Formatting, linting, tests, and contract fixtures pass.
2. Schema changes are intentional and compatibility is documented.
3. Processing changes include before/after benchmark evidence.
4. New models include source, license, size, hash, and platform validation.
5. User-visible behavior and recovery instructions are documented.
