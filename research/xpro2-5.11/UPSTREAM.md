# This branch against current upstream

Measured against `justjake/fujimod` `main` at `ceeedba2`; this branch starts from
`fb467534` and is 39 commits ahead of it, with 1,037 upstream commits in between.

## Nothing collides at file level

| Measure | Result |
|---|---|
| Files this branch changes | 156 |
| Files upstream changed since the merge base | 2,140 |
| Overlap | 75 files — a rebase would conflict there, not merge cleanly |

## What upstream already has

| This branch | Upstream | Consequence |
|---|---|---|
| `XipPage.data` allowed as dead | still a warning | lint-only difference |
| SMAPH CPU fallback (`677f1a81`) | `a948538b`, the same change | drop ours |
| `catalog/bodies/xpro2/body.json` (new) | no such file | ours to propose |
| `catalog/bodies/xpro2/controls.json` (1,438 lines, 27 connections) | 836 lines, the same 27 controls, no connections | ours is the fuller declaration |
| `RunScene::Xpp`, `PictureResources`, `set_picture`, `scene_sensor_image`, `liveViewSource` | absent | ours to propose |

## What upstream has that this branch lacks

- 75 block records under `catalog/soc/xpp/blocks/` (this branch has none) and the
  `device_for` / `block_register_layouts` / `gic_node` helpers that read them.
- A shared control path: `GpioInputs`, the rotary capability and the switch panel
  (`da01636b`, `0e13f7da`, `070b46b5`, `3749716e`, `3ed9b25c`).
- A `Camera` builder and one session loop for every host.

## How to carry the work over

1. **Windows build and tests** — independent of the X-Pro2; proposed first.
2. **Lint sweep** — independent; makes `clippy -D warnings` pass.
3. **Cortex-R5 controls** — independent; a one-file fix with 305 tests behind it.
4. **X-Pro2 catalog** — add `body.json` and the full `controls.json`, taking upstream's
   `lever` behaviour for `viewfinder_selector`, `battery_latch` and
   `battery_chamber_cover_latch`, which this branch models as a button or as mechanical.
5. **XPP picture and scene** — merge `crates/emulator-boot/src/xpp.rs` keeping both sides:
   upstream adds `device_for`, `block_register_layouts`, `gic_node`; this branch adds
   `scene_sensor_image`, `resolved_definition`, `soc_profile`.

Steps 1 to 3 are the branches `contrib/crate-graph`, `contrib/clippy` and
`contrib/cortex-r5`. Steps 4 and 5 need a decision from the maintainer first: the
catalog layout and the control path are both under change upstream.
