"""Press keys on an X-Pro2 and show the screen digest after each one.

This is how the input path was shown to work: idle stop must be off, or every queued
contact stays in the queue and the guest stays asleep. The digest of the published
display words moves whenever the screen changes.
"""
import json

import fujimod._native as native

import support

KEYS = [
    "menu_ok_button",
    "selector.down",
    "selector.down",
    "menu_ok_button",
    "disp_back_button",
]


def main():
    dat, inverted, ffboot = support.assets()
    board = support.boot(native, dat, inverted, ffboot)
    reference = json.loads(native.catalog.resolve_configuration("xpro2-511"))["reference"]
    problems = board.connect_controls(json.dumps(reference), support.TICKS)
    print(f"control problems: {problems}", flush=True)
    # The guest idles between inputs; without this the contacts never arrive.
    board.set_idle_stop(False)
    board.run_for_instructions(20_000_000, support.QUANTUM)
    print(f"ready {support.digest(board)}", flush=True)
    for key in KEYS:
        board.contact_input(key, None, False)
        board.contact_input(key, None, True)
        board.run_for_instructions(20_000_000, support.QUANTUM)
        queue = json.loads(board.control_state())["inputQueue"]
        print(
            f"{key} {support.digest(board)} completed={queue['completed']}"
            f" now={board.now}",
            flush=True,
        )
    support.report_transfers(board)


main()