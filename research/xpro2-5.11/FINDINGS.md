# Findings

Three things about the X-Pro2 in this emulator are not obvious from the code. Each was
measured on this branch; the numbers below are what the emulator printed.

## 1. Idle stop blocks physical input

After boot the firmware waits in an idle loop. With `set_idle_stop(true)` (the setting
the boot probes used) every `run_for_instructions` returns after roughly twelve ticks,
so a queued contact never reaches the guest:

```
booted              now= 48,496,710  idle=True   inputQueue completed=0
board.set_idle_stop(False)
idle 0              now=101,957,152  idle=False  completed=0
press menu_ok       now=310,299,463  idle=False  completed=2   <- delivered
press selector.down now=582,651,507  idle=False  completed=4
```

Turn idle stop off after `connect_controls` and the firmware runs its key scanner.
The screen proves it: the digest of `device_states("xpp-display-output")[0]["words"]`
changes with each key.

```
ready            a8b116948d8b57ec  115 words
menu_ok_button   daa7afa6a3ee62ca  116   menu opens
selector.down    537d92d65e4b91ff  116   the selection moves
menu_ok_button   386b768858d33210  114   a submenu opens
disp_back_button 7214016609ea25b7  113   it closes
```

Counting words alone hides this: the count moves by one or two, the digest moves every
time. `scripts/menu.py` prints it.

## 2. The I2C bus is observable, and the firmware uses bus 1

`device_states("designware-i2c")` returns each controller with its child bus state;
`bus.transfers` holds the last 64 transfers as `{address, read, bytes}`.

A 934,445,098-tick run with menu, VIEW MODE and viewfinder input produced:

```
bus 0: no transfers at all
bus 1: addresses 0x1c (accelerometer), 0x36, 0x39 (eye sensor)
```

The X-Pro2 discovery therefore declares the eye sensor at `0x39` on bus 1, and answers
its register reads. `0x44` on bus 0 stays declared from the firmware's descriptor table
(`0xd7fdbb8`, device 3) but the firmware never addresses it in any state this boot path
reaches.

`scripts/i2c.py` prints the transfers per bus.

## 3. A Cortex-R5 cannot take the A-profile translation controls

Every test that opened `Model::CortexR5F` died with `0xC0000005` on Windows while the
Cortex-A7 ran normally. Probing `Library::open` one call at a time showed two causes,
both A-profile only:

| Call | Symptom |
|---|---|
| `extensions.page_hooks(raw, true)` | faults while enabling: the extension walks a page table an R5 does not have |
| `uc_ctl(ctl(13, 1, 1), cache_bytes)` | returns, then the first `uc_mem_map` faults on the state it left |

An R5 translates through PMSA regions. Skipping both controls for `CortexR5F` takes
`fujimod-emulator-cpu-unicorn` from a crash to 305 passing tests.

## 4. Why task 9 is blocked

Watching the whole flash window (`0xf0000000..0xf4000000`) across 1,600 boot rounds
that reach quiescence:

```
booted now=48,497,910
distinct 64 KiB flash buckets read: 1   (0xf3820000, two words)
writes: 0
```

The boot this branch runs loads `ffboot.bin` into DRAM and executes it; it never reads
the flash device, so the settings block is never touched where a watch could see it.
Reaching it needs one of: the original NOR boot, which needs the SoC boot ROM (this
repository ships records for two other bodies and none for the X-Pro2); a dump from a
physical body; or locating the offsets in the image by hand.
