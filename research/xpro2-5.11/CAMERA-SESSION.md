# Starting a session with the camera

This is the checklist for the next section: a machine that can drive a physical X-Pro2.
Everything here is read-only on the body until a step says otherwise.

## 1. Get the checkout running

```sh
git clone https://github.com/paulnorinn/fujimod.git
cd fujimod
git switch paulnorinn/xpro2-serial7-irq
git submodule update --init crates/emulator-cpu-unicorn/vendor/unicorn
mise install && pnpm install
pnpm --filter fujimod run setup:tooling
python scripts/python_extension.py
```

On Windows the build needs `CC`/`CXX` pointing at `clang-cl.exe`, plus `PKG_CONFIG`,
`PKG_CONFIG_PATH`, `PROTOC` and `PROTOC_INCLUDE`; `CLANG` and `LD_LLD` are found for you.
`reports/handover-2026-11-25.md` has the exact values and the traps met so far.

## 2. Prove the emulator still reproduces this branch

Point the scripts at local copies of the update and the SD boot image:

```sh
export FUJIMOD_XPRO2_DAT=/path/to/FWUP0008.DAT
export FUJIMOD_FFBOOT=/path/to/ffboot.bin
python research/xpro2-5.11/scripts/boot.py
```

Expected, and a stop sign if any of it differs:

```
display0 words count: 113
battery +0x00=0x0 +0x14=1 +0x98(adc2)=700
now=48496710 quiescent=True
bus 0: 0 transfers
bus 1: 41 transfers, addresses ['0x1c', '0x36', '0x39']
```

```sh
python research/xpro2-5.11/scripts/menu.py
```

Expected digests, in order: `a8b1...` (ready), `daa7...` (menu opens), `537d...`
(selection moves), `386b...` (a submenu opens), `7214...` (it closes). If the digests
move but the transfers stay on bus 1, nothing has regressed.

## 3. Collect from the body (read-only)

| Order | What | Why |
|---|---|---|
| 1 | Firmware version and build, to confirm 5.11 | confirms the configuration id `xpro2-511` |
| 2 | The settings store, per the update handover's section 3.5 | the layout for `settings.generated.json` and `first-start.generated.json` (task 9) |
| 3 | `0xe87592`, `0xe87594`, `0xe875d4` with the eyepiece covered and uncovered | whether the firmware keeps eye state there at all |
| 4 | The I2C controller registers while the eye-sensor menu page is open | whether the body addresses `0x44` on bus 0 at all |
| 5 | Settings keys `0x11a2`, `0x11a4`, `0x11a9`, `0x11ac` | compare with the firmware's defaults `0x08`, `0x40`, `0x04`, `0x03` |

If a value in step 5 differs from the default, that is a user setting: the model must
carry it, not the default.

## 4. Where the results go

- Measured eye-sensor replies: the `i2c_parts` entry at address `0x39`, bus 1, in
  `configurations/emulator/xpro2-discovery/hardware-inputs.json`. Its `basis` field
  must say what was measured and what was not; the neighbouring entries show the form.
- A settings layout: `catalog/firmware/xpro2/5.11/settings.generated.json`. That file is
  generated, so change its inputs or its generator; do not write it by hand.
- Anything that changes the nine-task status: `README.md` in this directory, plus a new
  section in `reports/status-2026-11-21.md` recording the measurement and the command.

## 5. What not to do

- Do not hand-edit a `*.generated.*` file.
- Do not write a value into the catalog that was not measured or derived from firmware
  evidence; the `basis` field exists so the difference stays visible.
- Do not commit firmware, ROM dumps, memory snapshots, photographs, serial numbers or
  calibration data.
