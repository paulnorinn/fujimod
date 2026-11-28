"""Boot an X-Pro2 5.11 to quiescence and print the regression markers.

These are the values this branch treats as its regression: the display output has
113 words, the battery status word +0x14 reads 1, and the ADC band is 700.
"""
import struct

import fujimod._native as native

import support


def main():
    dat, inverted, ffboot = support.assets()
    board = support.boot(native, dat, inverted, ffboot)
    words = __import__("json").loads(board.device_states("xpp-display-output"))[0]["words"]
    print(f"display0 words count: {len(words)}", flush=True)
    battery = bytes(board.mem_read(0, 0x10BCF3C, 0xD0))
    print(
        f"battery +0x00=0x{struct.unpack_from('<H', battery, 0)[0]:x}"
        f" +0x14={battery[0x14]}"
        f" +0x98(adc2)={struct.unpack_from('<H', battery, 0x98)[0]}",
        flush=True,
    )
    eye = bytes(board.mem_read(0, 0xE87590, 0x50))
    print(
        f"eye sensor words: 0xe87592={eye[2]} 0xe87594={eye[4]} 0xe875d4={eye[0x44]}",
        flush=True,
    )
    print(f"now={board.now} quiescent={board.quiescent}", flush=True)
    support.report_transfers(board)


main()