import argparse
from dataclasses import dataclass
from math import ceil, floor, log2
import sys
from typing import BinaryIO

BITS_IN_BYTE = 8
MIN_BYTE_VALUE = 0x00
MAX_BYTE_VALUE = 0xFF
BYTES_TO_READ_AT_ONCE = 4096


@dataclass
class LzssConfig:
    window_size: int  # bytes
    length_width: int  # bits
    length_bias: int

    distance_width: int = 0  # bits, 0 means auto
    flag_width: int = 1  # bits
    flag_zero_means_literal: bool = True
    distance_from_end: bool = False


class BitBuffer:
    def __init__(self):
        self._buffer = b''
        self._offset = 0
        self._remaining_bits = 0
    
    def add_offset(self, offset: int):
        self._offset += offset
        self._remaining_bits -= offset
        removed_byte_count = floor(self._offset / BITS_IN_BYTE)
        self._buffer = self._buffer[removed_byte_count:]
        self._offset -= removed_byte_count * BITS_IN_BYTE
    
    def add_bytes(self, b: bytes):
        self._buffer += b
        self._remaining_bits += len(b) * BITS_IN_BYTE

    @property
    def buffer(self):
        return self._buffer

    @property
    def offset(self):
        return self._offset

    @property
    def remaining_bits(self):
        return self._remaining_bits


class SlidingWindow:
    def __init__(self, size: int, fill: int = MIN_BYTE_VALUE, distance_from_end=False):
        assert MIN_BYTE_VALUE <= fill <= MAX_BYTE_VALUE
        self._buffer = bytearray([fill] * size)
        self._first = 0
        self._distance_from_end = distance_from_end
        self._inserted_count = 0
    
    def insert(self, character: int):
        assert MIN_BYTE_VALUE <= character <= MAX_BYTE_VALUE
        self._buffer[self._first] = character
        self._first += 1
        if self._first >= len(self._buffer):
            self._first = 0
        if self._inserted_count < len(self._buffer):
            self._inserted_count += 1

    def at(self, position: int, length: int) -> bytearray:
        if self._distance_from_end:
            assert 0 <= length <= self._inserted_count
            position += self._first - 1
            if position < 0:
                position += len(self._buffer)
            if position >= len(self._buffer):
                position -= len(self._buffer)
            to_end = self._buffer[position:(position + length)]
            remaining_length = length - len(to_end)
            if remaining_length == 0:
                return to_end
            return to_end + self._buffer[:remaining_length]
        else:
            assert 0 <= length <= len(self._buffer)
            position += self._first
            if position >= len(self._buffer):
                position -= len(self._buffer)
            to_end = self._buffer[position:(position + length)]
            remaining_length = length - len(to_end)
            if remaining_length == 0:
                return to_end
            return to_end + self._buffer[:remaining_length]


def bits_from_byte(value: int, offset: int, length: int) -> int:
    '''
    Reads `length` bits from a single byte (`value`), skipping the first `offset` bits.
    Read bits are converted to an unsigned integer.
    :param value: Source byte.
    :param offset: Offset in bits.
    :param length: Length in bits.
    :return: Unsigned integer in range [0; 2^`length`)
    '''
    assert MIN_BYTE_VALUE <= value <= MAX_BYTE_VALUE
    assert offset + length <= BITS_IN_BYTE
    offset_mask = MAX_BYTE_VALUE >> offset
    shift = BITS_IN_BYTE - offset - length
    return (value & offset_mask) >> shift

def bits_from_bytes(buffer: BitBuffer, length: int) -> int:
    '''
    Assembles an unsigned integer using `length` bits from a managed sequence of bytes (`buffer`), skipping offset if needed.
    The integer is read in big endian order.
    :param buffer: Source bytes (with offset).
    :param length: Length in bits.
    :return: Unsigned integer in range [0; 2^`length`)
    '''
    buf = buffer.buffer
    offset = buffer.offset
    bytes_needed = ceil(length / BITS_IN_BYTE) + (1 if offset > 0 else 0)
    assert len(buf) >= bytes_needed
    buffer.add_offset(length)

    pre_bits = min(BITS_IN_BYTE - offset, length)
    value = bits_from_byte(buf[0], offset, pre_bits)
    if pre_bits == length:
        return value

    buf = buf[1:]
    full_bytes = floor((length - pre_bits) / BITS_IN_BYTE)
    post_bits = length - pre_bits - full_bytes * BITS_IN_BYTE
    for _ in range(full_bytes):
        value <<= BITS_IN_BYTE
        value += buf[0]
        buf = buf[1:]
    if post_bits == 0:
        return value

    value <<= post_bits
    value += bits_from_byte(buf[0], 0, post_bits)
    return value


def decode(input_file: BinaryIO, output_file: BinaryIO, config: LzssConfig):
    buffer = BitBuffer()
    is_literal = (lambda flag: flag == 0) if config.flag_zero_means_literal else (lambda flag: flag != 0)
    if config.distance_width < 1:
        config.distance_width = ceil(log2(config.window_size))
    literal_code_word_width = config.flag_width + BITS_IN_BYTE
    reference_code_word_width = config.flag_width + config.length_width + config.distance_width
    min_code_word_width = min(literal_code_word_width, reference_code_word_width)
    max_code_word_width = max(literal_code_word_width, reference_code_word_width)

    # read the first literal
    buffer.add_bytes(input_file.read(ceil(literal_code_word_width / BITS_IN_BYTE)))
    flag = bits_from_bytes(buffer, config.flag_width)
    assert is_literal(flag)
    first_character = bits_from_bytes(buffer, BITS_IN_BYTE)
    window = SlidingWindow(config.window_size, first_character)
    output_file.write(bytes([first_character]))

    while True:
        if buffer.remaining_bits < max_code_word_width:
            buffer.add_bytes(input_file.read(BYTES_TO_READ_AT_ONCE))
        if buffer.remaining_bits < min_code_word_width:
            return  # EOF
        flag = bits_from_bytes(buffer, config.flag_width)
        if is_literal(flag):
            literal = bits_from_bytes(buffer, BITS_IN_BYTE)
            window.insert(literal)
            output_file.write(bytes([literal]))
        else:
            distance = bits_from_bytes(buffer, config.distance_width)
            length = bits_from_bytes(buffer, config.length_width)
            length += config.length_bias
            output_file.write(window.at(distance, length))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='LZSS decoder')
    parser.add_argument('input_file', type=argparse.FileType('rb'), nargs='?', help='Input file (to be decoded)')
    parser.add_argument('output_file', type=argparse.FileType('wb'), nargs='?', help='Output file (target)')
    args = parser.parse_args()

    input_file = args.input_file if args.input_file is not None else sys.stdin.buffer
    output_file = args.output_file if args.output_file is not None else sys.stdout.buffer
    # input_file = open(r'D:\Programowanie\studia\KODA\koda-lzss\py\examples\aaaaaaaaaaaaaaa.lzss', 'rb')
    # output_file = sys.stdout.buffer
    config = LzssConfig(256, 8, 3, flag_zero_means_literal=False)
    decode(input_file, output_file, config)
 