# Quality Evaluation Findings

## SAM candidate selection is not yet a reliable quality signal

The first licensed Open Images smoke case contains a crowded martial-arts scene
and an instance mask for the boy on the left. A positive point at relative
coordinates `(0.15, 0.60)` is unambiguously inside the annotated lower torso.

The current product policy chooses the decoder candidate with the highest
model-predicted IoU. On this case it selects a partial mask:

- selected candidate predicted IoU: approximately 0.77;
- selected candidate actual IoU against the human mask: approximately 0.47;
- alternate candidate predicted IoU: approximately 0.57;
- alternate candidate actual IoU: approximately 0.96.

The checksum-pinned [quality smoke report](../reports/quality-smoke-20260714T073357Z.md)
records the selected candidate from clean commit `f78e46b`.

This is evidence that model-reported IoU is useful as a confidence hint but is
not a ground-truth quality measurement. It also suggests that candidate
selection may need an area/stability heuristic or a retry strategy.

No selection-policy change is accepted from this single case. The next quality
work should expose per-candidate predicted IoU, mask area, and actual fixture
IoU in the evaluator, then compare policies over at least the 20 required
segment prompts. This avoids overfitting the product to one crowded image.
