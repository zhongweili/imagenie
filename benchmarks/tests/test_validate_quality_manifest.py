from __future__ import annotations

import hashlib
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from validate_quality_manifest import validate  # noqa: E402


class QualityManifestValidationTests(unittest.TestCase):
    def manifest(self, root: pathlib.Path) -> dict:
        (root / "input.bin").write_bytes(b"input")
        (root / "reference.bin").write_bytes(b"reference")
        digest = lambda value: hashlib.sha256(value).hexdigest()
        return {
            "schema_version": "1.0",
            "gate_status": "blocked",
            "gate_blockers": ["fixture set is intentionally incomplete"],
            "minimum_cases": {operator: 1 for operator in (
                "rembg", "upscale", "restore", "segment", "erase", "segment_erase"
            )},
            "approved_sources": [{
                "id": "fixture-source",
                "homepage": "https://example.com/source",
                "annotation_license": {"name": "CC-BY-4.0", "url": "https://example.com/a"},
                "expected_image_license": {"name": "CC-BY-2.0", "url": "https://example.com/i"},
                "per_asset_verification_required": True,
            }],
            "cases": [{
                "id": "fixture-rembg",
                "operator": "rembg",
                "source": {
                    "dataset_id": "fixture-source",
                    "image_id": "one",
                    "landing_url": "https://example.com/one",
                    "author": "Fixture Author",
                    "license": {"name": "CC-BY-2.0", "url": "https://example.com/license"},
                },
                "input": {"path": "input.bin", "sha256": digest(b"input")},
                "reference": {
                    "path": "reference.bin",
                    "sha256": digest(b"reference"),
                    "annotation_method": "fixture truth",
                },
                "metrics": ["iou"],
            }],
        }

    def test_valid_blocked_manifest_may_be_incomplete(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            self.assertEqual(validate(self.manifest(root), root), [])

    def test_checksum_mismatch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            manifest = self.manifest(root)
            manifest["cases"][0]["input"]["sha256"] = "0" * 64
            self.assertTrue(any("does not match" in error for error in validate(manifest, root)))

    def test_release_check_rejects_blocked_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            errors = validate(self.manifest(root), root, require_ready=True)
            self.assertIn("quality gate is not ready", errors)


if __name__ == "__main__":
    unittest.main()
