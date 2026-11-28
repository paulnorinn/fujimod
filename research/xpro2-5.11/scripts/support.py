"""Shared setup for the X-Pro2 probes.

The firmware update and the SD boot image are not part of this repository.
Point FUJIMOD_XPRO2_DAT and FUJIMOD_FFBOOT at local copies.
"""
import hashlib
import json
import os
import sys
from pathlib import Path

QUANTUM = 500_000
TICKS = 48_000_000


def assets():
    dat_path = os.environ.get("FUJIMOD_XPRO2_DAT")
    ffboot_path = os.environ.get("FUJIMOD_FFBOOT")
    if not dat_path or not ffboot_path:
        sys.exit(
            "set FUJIMOD_XPRO2_DAT and FUJIMOD_FFBOOT to the X-Pro2 5.11 update"
            " and the SD boot image, neither of which is in this repository"
        )
    dat = Path(dat_path).read_bytes()
    return dat, bytes(b ^ 0xFF for b in dat[604:]), Path(ffboot_path).read_bytes()


def boot(native, dat, inverted, ffboot):
    """Boot an X-Pro2 and stop once the guest is quiescent."""
    definition = json.loads(
        native.emulator.xpp_definition(
            "xpro2-511",
            json.dumps([[["Pc", 0]], [["Pc", 0]]]),
            assets={
                "ffboot.bin": ffboot,
                "xt20-211-statistics-window": bytes(0x100000),
                "xpro2-511-update": dat,
                "xpro2-511-flash": inverted,
            },
        )
    )
    payload = {definition["flash_asset"]: inverted} if definition.get("flash_asset") else {}
    board = native.emulator.Board(
        json.dumps(definition["execution"]),
        json.dumps(definition["devices"]),
        "{}",
        payload,
        {},
        execution_mode="strict",
        continue_refusals=False,
        engine="unicorn",
    )
    board.mem_write(0, 0, ffboot)
    board.set_idle_stop(True)
    for _ in range(1500):
        board.run_for_instructions(25_000_000, QUANTUM)
    return board


def digest(board):
    """A digest of the published display words: it moves whenever the screen does."""
    words = json.loads(board.device_states("xpp-display-output"))
    payload = json.dumps([entry["words"] for entry in words]).encode()
    return hashlib.sha256(payload).hexdigest()[:16], len(words[0]["words"])


def report_transfers(board):
    """Print every I2C transfer the guest has made, per controller."""
    controllers = json.loads(board.device_states("designware-i2c"))
    for index, controller in enumerate(controllers):
        bus = controller.get("bus") or {}
        entries = bus.get("transfers") or []
        addresses = [hex(address) for address in sorted({e["address"] for e in entries})]
        print(f"bus {index}: {len(entries)} transfers, addresses {addresses}", flush=True)
