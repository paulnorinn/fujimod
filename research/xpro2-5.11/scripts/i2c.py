"""Print every I2C transfer the X-Pro2 boot makes, per controller.

The controllers are the DesignWare model: device_states("designware-i2c") returns one
entry per controller, and each child bus keeps its last 64 transfers.
"""
import fujimod._native as native

import support


def main():
    dat, inverted, ffboot = support.assets()
    board = support.boot(native, dat, inverted, ffboot)
    print(f"booted now={board.now}", flush=True)
    support.report_transfers(board)


main()