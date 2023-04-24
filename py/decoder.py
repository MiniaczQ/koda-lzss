import argparse
from dataclasses import dataclass
from math import ceil, floor, log2
import sys
from typing import BinaryIO


@dataclass
class LzssConfig:
    window_size: int  # bytes
    length_width: int  # bits
    length_bias: int

    distance_width: int = 0  # bits, 0 means auto
    flag_width: int = 1  # bits
    flag_zero_means_literal: bool = True
    distance_from_end: bool = False


def bit_extract(value: int, offset: int, length: int) -> int:
    assert 0 <= value <= 255
    assert offset + length <= 8
    offset_mask = 0xFF >> offset
    shift = 8 - offset - length
    return (value & offset_mask) >> shift


def read_value(buffer: bytes, bit_width: int, bit_offset: int) -> int:
    bytes_needed = ceil(bit_width / 8) + (1 if bit_offset > 0 else 0)
    assert len(buffer) >= bytes_needed

    pre_bits = min(8 - bit_offset, bit_width)
    value = bit_extract(buffer[0], bit_offset, min(8 - bit_offset, bit_width))
    if pre_bits == bit_width:
        return value

    buffer = buffer[1:]
    full_bytes = floor((bit_width - pre_bits) / 8)
    post_bits = bit_width - pre_bits - full_bytes * 8
    for _ in range(full_bytes):
        value <<= 8
        value += buffer[0]
        buffer = buffer[1:]
    if post_bits == 0:
        return value

    value <<= post_bits
    value += bit_extract(buffer[0], 0, post_bits)
    return value


class SlidingWindow:
    def __init__(self, size: int, fill: int = ord('\0'), distance_from_end=False):
        assert 0 <= fill <= 255
        self._buffer = bytearray([fill] * size)
        self._first = 0
        self._distance_from_end = distance_from_end
        self._inserted_count = 0
    
    def insert(self, character: int):
        assert 0 <= character <= 255
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


def add_offset(added_offset: int, buffer: bytes, offset: int, remaining_bits: int) -> tuple[bytes, int, int]:
    offset += added_offset
    remaining_bits -= added_offset
    removed_byte_count = floor(offset / 8)
    buffer = buffer[removed_byte_count:]
    offset -= removed_byte_count * 8
    return (buffer, offset, remaining_bits)


def decode(input_file: BinaryIO, output_file: BinaryIO, config: LzssConfig):
    buffer = b''
    bit_offset = 0
    remaining_bits = 0
    if config.distance_width < 1:
        config.distance_width = ceil(log2(config.window_size))
    literal_code_word_width = config.flag_width + 8
    reference_code_word_width = config.flag_width + config.length_width + config.distance_width
    min_code_word_width = min(literal_code_word_width, reference_code_word_width)
    max_code_word_width = max(literal_code_word_width, reference_code_word_width)

    # read the first literal
    buffer += input_file.read(ceil(literal_code_word_width / 8))
    remaining_bits += len(buffer) * 8
    flag = read_value(buffer, config.flag_width, 0)
    (buffer, bit_offset, remaining_bits) = add_offset(config.flag_width, buffer, bit_offset, remaining_bits)
    assert (flag == 0) == config.flag_zero_means_literal
    first_character = read_value(buffer, 8, bit_offset)
    (buffer, bit_offset, remaining_bits) = add_offset(8, buffer, bit_offset, remaining_bits)
    window = SlidingWindow(config.window_size, first_character)
    output_file.write(window.at(0, 1))

    while True:
        if len(buffer) < max_code_word_width:
            len_before_read = len(buffer)
            buffer += input_file.read(4096)
            len_after_read = len(buffer)
            remaining_bits += (len_after_read - len_before_read) * 8
        if remaining_bits < min_code_word_width:
            return  # EOF
        flag = read_value(buffer, config.flag_width, bit_offset)
        (buffer, bit_offset, remaining_bits) = add_offset(config.flag_width, buffer, bit_offset, remaining_bits)
        if (flag == 0) == config.flag_zero_means_literal:
            literal = read_value(buffer, 8, bit_offset)
            (buffer, bit_offset, remaining_bits) = add_offset(8, buffer, bit_offset, remaining_bits)
            window.insert(literal)
            output_file.write(window.at(config.window_size - 1, 1))
        else:
            distance = read_value(buffer, config.distance_width, bit_offset)
            (buffer, bit_offset, remaining_bits) = add_offset(config.distance_width, buffer, bit_offset, remaining_bits)
            length = read_value(buffer, config.length_width, bit_offset)
            (buffer, bit_offset, remaining_bits) = add_offset(config.length_width, buffer, bit_offset, remaining_bits)
            length += config.length_bias
            output_file.write(window.at(distance, length))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='LZSS decoder')
    parser.add_argument('input_file', type=argparse.FileType('rb'), nargs='?', help='Input file (to be decoded)')
    parser.add_argument('output_file', type=argparse.FileType('wb'), nargs='?', help='Output file (target)')
    args = parser.parse_args()

    input_file = args.input_file if args.input_file is not None else sys.stdin.buffer
    output_file = args.output_file if args.output_file is not None else sys.stdout.buffer
    config = LzssConfig(256, 8, 3, flag_zero_means_literal=False)
    decode(input_file, output_file, config)
