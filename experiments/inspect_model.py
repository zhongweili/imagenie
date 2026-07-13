"""Inspect ONNX model metadata: inputs, outputs, producer, and architecture hints."""

import sys

import onnx


def inspect(path: str) -> None:
    model = onnx.load(path, load_external_data=False)
    print(f"=== {path} ===")
    print(f"producer: {model.producer_name} {model.producer_version}")
    print(f"ir_version: {model.ir_version}, opset: {model.opset_import[0].version}")
    if model.metadata_props:
        for prop in model.metadata_props:
            print(f"metadata: {prop.key} = {prop.value[:200]}")
    if model.doc_string:
        print(f"doc: {model.doc_string[:200]}")

    graph = model.graph
    print(f"graph name: {graph.name}")
    for inp in graph.input:
        shape = [
            d.dim_value if d.dim_value else d.dim_param
            for d in inp.type.tensor_type.shape.dim
        ]
        dtype = onnx.TensorProto.DataType.Name(inp.type.tensor_type.elem_type)
        print(f"input:  {inp.name} {dtype} {shape}")
    for out in graph.output:
        shape = [
            d.dim_value if d.dim_value else d.dim_param
            for d in out.type.tensor_type.shape.dim
        ]
        dtype = onnx.TensorProto.DataType.Name(out.type.tensor_type.elem_type)
        print(f"output: {out.name} {dtype} {shape}")

    op_counts: dict[str, int] = {}
    for node in graph.node:
        op_counts[node.op_type] = op_counts.get(node.op_type, 0) + 1
    top_ops = sorted(op_counts.items(), key=lambda x: -x[1])[:15]
    print(f"nodes: {len(graph.node)}, top ops: {top_ops}")

    # Node name prefixes often reveal the original PyTorch module names
    prefixes: dict[str, int] = {}
    for node in graph.node:
        if node.name and "/" in node.name:
            prefix = node.name.strip("/").split("/")[0]
            prefixes[prefix] = prefixes.get(prefix, 0) + 1
    if prefixes:
        top_prefixes = sorted(prefixes.items(), key=lambda x: -x[1])[:10]
        print(f"node name prefixes: {top_prefixes}")
    print()


if __name__ == "__main__":
    for p in sys.argv[1:]:
        inspect(p)
