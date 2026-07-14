# Handoff: imagenie CLI 改造工程开发

> 交接自 2026-07-13 规划+实验 session。本文档给下一个 session 提供完整上下文。开发完成后本文档可删除。

## 进度（2026-07-13 更新）

- ✅ **PR1 已完成**（commit `6ce837e`）：workspace 化 + `imagenie-core` 抽取，desktop 行为零变化已验证
- ✅ **PR2 已完成**（commit `e554cfc`）：`imagenie` CLI 五算子 + models/info/capabilities + `--json` 事件流，click-to-erase 端到端验证通过
- ⬜ PR3：cargo-dist + Homebrew tap + README 重写
- ⬜ PR4：desktop commands 层改调 core（rembg 的 apply_mask 逻辑目前在 CLI 和 desktop 各有一份，PR4 应下沉到 core）

2026-07-14 后续推进：
- 🚧 **v0.1 Agent Contract + Eval**：已新增产品规格、路线图、JSONL v1 schema、版本化 capabilities、CLI 契约测试、benchmark manifest 和贡献/PR/Issue 模板。
- ✅ **Core API 第一阶段**：新增 `ImageOperator`/`ParameterizedImageOperator`/`ProcessOutput`；CLI 五算子均走 core facade，desktop 的 upscale/restore/rembg 同步迁移；rembg alpha 合成和 erase mask 膨胀下沉 core，SAM operator 会按输入路径/大小/mtime 复用 embedding。
- ✅ **性能 baseline 第一阶段**：新增隔离进程 release-mode runner，在 M3 Max/CPU EP 上记录了六个算子的 clean-commit cold/warm/RSS 基线；SAM 热分割约 28ms，当前最大风险是 rembg-high 峰值约 11GiB 和 upscale 热延迟约 19.2s。
- 下一步补齐有明确许可和标注的质量数据集，同时 profile rembg-high/upscale，再按不低于 20% 的量化目标做 runtime/CoreML、分辨率或 tiling 优化，最后进入 cargo-dist 发布。

PR1/PR2 实施要点（后续 session 需要知道的）：
- ort 已升 **2.0.0-rc.12**：`Session::run` 改为 `&mut self`，`ModelProcessor` 用 Mutex 包 session（推理串行，预/后处理保持 rayon 并行）
- LaMa Einsum / SAM encoder 在 Rust ort 验证可跑；LaMa CPU ~4s/张，SAM encode ~1s、decode ~20ms
- 模型分发已定：现有 3 个走自家 GitHub Release；BiRefNet 走 rembg release URL；SAM（zip，pull 时解包）/LaMa 走 HF；sha256 全部在 `crates/imagenie-core/src/registry.rs`
- `index.html` 曾在 landing page 拆分时被误删，PR1 已恢复
- 本地开发：实验模型已软链进 `~/.imagenie/models/`（sam/erase/rembg-hq 不用重新下载）
- CoreML EP benchmark 未做，遗留到 PR3 之后

## 项目定位

把 imagenie 从 Tauri 桌面 app 改造为 **agent-native 本地图像处理 CLI**。

一句话定位：**"privacy-first image processing for your terminal and your agents"**

背景：2026-07 调研确认这是结构性空白——MCP/agent 生态的图像工具全部是云端 API 或纯格式转换（Sharp/ImageMagick 壳），没有一个项目做「本地 AI 推理 + 多任务 + agent 友好接口」。Rust 生态也只有单任务实验性 crate。

关键决策（已与 owner 确认）：
- 沿用现有 repo（408 stars 是资产），不新开
- CLI 优先，不做 MCP server（本地场景 CLI 更优；MCP 以后用薄壳包 CLI 即可）
- 模型全部用社区现成 ONNX，不自己导出转换
- 二进制名保留 `imagenie`

## 当前状态

- 本地 repo：`~/repos/imagenie`，工作分支 **`feat/cli`**（已建，从 master 切出）
- master 上有一个未 push 的提交 `f8a05de`（experiments/ 目录）
- 实验已完成，结论见 `experiments/RESULTS.md`（**必读**，含每个模型的预处理参数和工程注意点）
- 实验产物在 `experiments/models/`、`experiments/images/`、`experiments/outputs/`（gitignored，可直接复用）

## v1 算子（5 个，模型全部验证通过）

| 算子 | 模型 | 来源 | 许可 | 备注 |
|---|---|---|---|---|
| rembg | 现有 ISNet（默认）+ BiRefNet-lite（`--quality high`） | 现有 release + rembg GitHub Releases | MIT | BiRefNet 输出是 logits 必须过 sigmoid；ImageNet mean/std；CPU 慢 5.6x |
| upscale | 现有 Real-ESRGAN f16 2x | 现有 release | - | 后续可从 OpenModelDB 换 4x |
| restore | 现有 GFPGAN 家族 | 现有 release | - | 512x512 固定输入 |
| segment | SAM 2.1 hiera_tiny（encoder 104MB + decoder 16MB） | HF `vietanhdev/segment-anything-2.1-onnx-models` | Apache 2.0 | encode ~585ms 一次，decode 12ms/点击 → 缓存 embedding |
| erase | LaMa fp32 | HF `Carve/LaMa-ONNX` | Apache 2.0 | 固定 512x512，需 tile/crop 策略；mask 需膨胀（实验用 r=12） |

Killer feature：**click-to-erase**（segment → dilate → erase 组合），端到端 ~1.8s CPU，已在 `experiments/test_segment_erase.py` 验证。

避雷：RMBG-2.0 是 CC BY-NC 非商用；IOPaint 已存档（其 LaMa mask 处理逻辑仍值得参考）；去噪任务无成熟社区 ONNX，已砍掉。

## 目标架构

```
imagenie/
├── Cargo.toml                 # [workspace]
├── crates/
│   ├── imagenie-core/         # 纯处理引擎（现 src-tauri/src/image/ 基本可直接迁入，无 Tauri 依赖）
│   │   └── src/               # model.rs processor.rs tensor.rs types.rs error.rs
│   │                          # + registry.rs（模型元数据+下载，从 commands/download.rs 抽出，
│   │                          #   进度回调改 impl Fn(Progress)）
│   └── imagenie-cli/          # 二进制名 `imagenie`
└── apps/desktop/              # 现有 Tauri app 挪入，commands/ 变 core 的薄包装
```

core 公共 API 收敛为一个入口：`process(op, inputs, params, callback) -> Vec<Result<Output>>`。

## CLI 设计（已定稿）

```
imagenie upscale  <INPUT>... [-o PATH] [--scale 2|4] [--format png|jpeg|webp]
imagenie rembg    <INPUT>... [-o PATH] [--quality fast|high] [--format png|webp]
imagenie restore  <INPUT>... [-o PATH]
imagenie segment  <INPUT> --point x,y [--box x1,y1,x2,y2] [-o mask.png]
imagenie erase    <INPUT> --mask MASK [-o PATH]
imagenie models   list | pull <NAME|all> | rm <NAME> | dir
imagenie info     <INPUT>... [--json]
imagenie capabilities --json    # 机器可读的算子/参数 schema，agent 发现用
```

约定：
- 多输入批处理：一次进程加载模型一次 + rayon 并行（解决冷启动）；`--jobs N` 默认 `min(4, cores)`；单张失败不中断整批
- 输出：单输入 `-o` 为文件；多输入 `-o` 为目录；省略则原图旁加后缀 `photo.upscaled.png`；默认不覆盖，`--force` / `--skip-existing`
- `-` = stdin/stdout 单图管道模式
- `--json`：stdout 输出 JSONL 事件流（start/done/error/summary），error 用稳定机器码；进度条走 stderr（非 TTY 自动关）
- 退出码：0 全成功 / 1 部分失败 / 2 参数错误 / 3 模型缺失 / 4 输入不可读
- 模型缺失时非 TTY 直接报错并给出确切命令（`run: imagenie models pull sam`），绝不静默下载；`--yes` 允许自动下载
- 模型目录：`dirs::data_dir()/imagenie/models/`（与现有 Tauri app 共用），`IMAGENIE_MODEL_DIR` 可覆盖，模型带 sha256 校验
- segment 的 `--json` 输出必须带 `iou` 和 mask 像素数（置信度信号，agent 判断"没点中→换点重试"；实验中点偏产出 iou 0.31/114px 垃圾 mask）

选型：clap v4 derive / indicatif / cargo-dist + Homebrew tap。

## PR 序列

1. **PR1**：workspace 化 + 抽 `imagenie-core`（desktop 路径引用 core，行为零变化）
2. **PR2**：`imagenie-cli` 五算子 + `models` 子命令 + `--json`
3. **PR3**：cargo-dist 发布链 + Homebrew tap + README 重写（新定位）
4. **PR4**：desktop 的 commands 层改调 core，删重复代码

## PR1 阶段待验证的技术风险

- **ort 升级**：现在用 `2.0.0-rc.9`，升到 2.0 稳定版
- **Einsum 支持**：LaMa 模型 17k 节点、Einsum-heavy，Python onnxruntime 已验证可跑，Rust ort 需确认（大概率没问题，都是 libonnxruntime，但要跑通）
- **CoreML EP**：实验只测了 CPU EP；加 CoreML feature flag 并 benchmark（BiRefNet 5.6x 慢的问题可能被 ANE 缓解）
- SAM encoder 在 ort 有 benign shape-merge warning，Rust 侧确认无碍即可
- 新模型（BiRefNet/SAM/LaMa）的分发：跟现有三个模型一样挂 GitHub Release，或直接引用上游 URL（rembg releases URL 稳定、HF 有 CDN），建议后者起步

## 实验代码索引（预处理逻辑可直接翻译成 Rust）

- `experiments/inspect_model.py` — ONNX 元数据检查工具
- `experiments/compare_rembg.py` — 两种抠图模型的 mean/std/sigmoid 预后处理
- `experiments/test_lama.py` — LaMa 的 image+mask 输入构造（输出已是 0-255，无需反归一化）
- `experiments/test_segment_erase.py` — SAM encoder/decoder 完整 IO + click-to-erase 链路（含 mask 膨胀）
- 参考实现：rembg 的 Python 源码（预后处理最清晰）、spandrel（超分 tile 策略）、IOPaint 源码（LaMa mask 膨胀/crop-around-mask）
