# X-Pro2 5.11 — what this checkout knows

This directory carries the X-Pro2 work of this branch: what was established, how to
reproduce it, and what still needs a physical camera. It is written for whoever
picks the work up next, on a machine that can drive a real X-Pro2.

## Status of the nine camera-free tasks

| # | Task | Status |
|---|---|---|
| 1 | Physical controls 31 of 36 | Established: the plan declares 36 inputs, 31 are supported and 5 are mechanical. The emulator delivers them to the firmware once idle stop is off (see `FINDINGS.md`). |
| 2 | `RunScene::Xpp` and `inputs.sensor_image` | Implemented on this branch. |
| 3 | Eye sensor `i2c0` | Declared from firmware evidence; the emulator shows the firmware polling `0x39` on bus 1 and never touching `0x44` on bus 0. Measured values still need a camera. |
| 4 | `Session::set_picture` for XPP | Implemented on this branch. |
| 5 | Live-view source notice | Implemented on this branch. |
| 6 | `camera-mount/build.rs` finds clang and ld.lld | Implemented on this branch. |
| 7 | libjpeg linked statically | Implemented on this branch. |
| 8 | Workspace tests and clippy on Windows | `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes, excluding the Tauri application. |
| 9 | `settings/first-start.generated.json` | Blocked. There is no generator in the repository, and the boot path this branch runs never reads the flash device (measured: two words). See `FINDINGS.md`. |

## Read next

1. `FINDINGS.md` — the three things worth knowing before touching the emulator:
   idle stop and physical input, reading the I2C bus, and the Cortex-R5 controls.
2. `UPSTREAM.md` — how this branch differs from current upstream, and the
   order in which work should be carried over.
3. `reports/status-2026-11-21.md` — the full record: every measurement, every log,
   every correction, in the order it was found.
4. `reports/handover-2026-11-25.md` — how to set up a Windows machine, and the
   read-only data to collect from a physical body.

## Scripts

The scripts need the firmware update and the SD boot image, which are not in this
repository. Point them at local copies:

```sh
export FUJIMOD_XPRO2_DAT=/path/to/FWUP0008.DAT
export FUJIMOD_FFBOOT=/path/to/ffboot.bin

python research/xpro2-5.11/scripts/boot.py       # boot to quiescent and print the regression markers
python research/xpro2-5.11/scripts/menu.py       # press keys and print the screen digest
python research/xpro2-5.11/scripts/i2c.py        # print every I2C transfer the boot makes
```

They install the extension from `bindings/python` first: `python scripts/python_extension.py`.

## What needs a camera

- Measured eye-sensor replies at `0x39` (bus 1): the declared zeros answer the firmware
  without refusal, so the shape is right, but the values are not measured.
- The settings store: a `settings.generated.json` layout needs the offsets inside the
  flash image, which neither the update file nor this boot path exposes.
- The eye-sensor settings keys `0x11a2`, `0x11a4`, `0x11a9` and `0x11ac` read from a
  body, to compare with the firmware's own defaults (`0x08`, `0x40`, `0x04`, `0x03`).
