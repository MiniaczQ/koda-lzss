import argparse
from dataclasses import dataclass
from math import ceil, floor, log2, sqrt
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
        self._total_bytes = 0
    
    def add_offset(self, offset: int):
        self._offset += offset
        self._remaining_bits -= offset
        removed_byte_count = floor(self._offset / BITS_IN_BYTE)
        self._buffer = self._buffer[removed_byte_count:]
        self._offset -= removed_byte_count * BITS_IN_BYTE
    
    def add_bytes(self, b: bytes):
        self._buffer += b
        self._remaining_bits += len(b) * BITS_IN_BYTE
        self._total_bytes += len(b)

    @property
    def buffer(self):
        return self._buffer

    @property
    def offset(self):
        return self._offset

    @property
    def remaining_bits(self):
        return self._remaining_bits

    @property
    def total_bytes(self):
        return self._total_bytes


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

    def insert_multiple(self, characters: bytes):
        for character in characters:
            self.insert(character)

    def at(self, position: int, length: int) -> bytearray:
        if self._distance_from_end:
            assert 0 <= length <= self._inserted_count
            position = self._first - 1 - position
            while position < 0:
                position += len(self._buffer)
            while position >= len(self._buffer):
                position -= len(self._buffer)
            to_end = self._buffer[position:(position + length)]
            remaining_length = length - len(to_end)
            if remaining_length == 0:
                return to_end
            return to_end + self._buffer[:remaining_length]
        else:
            assert 0 <= length <= len(self._buffer)
            position += self._first
            while position >= len(self._buffer):
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

def print_debug(index: int, symbol: tuple[int] | tuple[int, int], config: LzssConfig, window: SlidingWindow | None):
    is_literal = len(symbol) == 1
    flag = 1 if (is_literal != config.flag_zero_means_literal) else 0

    print(f'#{index}', file=sys.stderr)
    symbol_with_flag = (flag, chr(symbol[0])) if is_literal else ((flag, ) + symbol)
    print(f' symbol:       {symbol_with_flag}', file=sys.stderr)
    characters = bytes([symbol[0]]) if is_literal else \
        window.at(symbol[0], symbol[1]) if window is not None else bytes([])
    print(f' characters:   {characters.decode("ansi", "ignore")}', file=sys.stderr)
    if window is not None:
        if config.window_size < 40:
            phrase = window.at(config.window_size if config.distance_from_end else config.window_size - config.window_size, config.window_size)
        else:
            start_phrase = window.at(config.window_size - 20 if config.distance_from_end else 20, 20)
            end_phrase = window.at(20 if config.distance_from_end else config.window_size - 20, 20)
            phrase = start_phrase + b'...' + end_phrase
        print(f' window[:20..-20:]: {phrase.decode("ansi", "ignore")}', file=sys.stderr)
    print(file=sys.stderr)


def decode(input_file: BinaryIO, output_file: BinaryIO, config: LzssConfig, debug=False):
    debug_index = 0
    buffer = BitBuffer()
    is_literal = (lambda flag: flag == 0) if config.flag_zero_means_literal else (lambda flag: flag != 0)
    if config.distance_width < 1:
        config.distance_width = ceil(log2(config.window_size))
    literal_code_word_width = config.flag_width + BITS_IN_BYTE
    reference_code_word_width = config.flag_width + config.length_width + config.distance_width
    min_code_word_width = min(literal_code_word_width, reference_code_word_width)
    max_code_word_width = max(literal_code_word_width, reference_code_word_width)
    if debug:
        print(f'Code word width: [{min_code_word_width}, {max_code_word_width}]\n', file=sys.stderr)

    # read the first literal
    buffer.add_bytes(input_file.read(ceil(literal_code_word_width / BITS_IN_BYTE)))
    flag = bits_from_bytes(buffer, config.flag_width)
    assert is_literal(flag)
    first_character = bits_from_bytes(buffer, BITS_IN_BYTE)
    if debug:
        print_debug(debug_index, (first_character, ), config, None)
        debug_index += 1
    window = SlidingWindow(config.window_size, first_character, config.distance_from_end)
    output_file.write(bytes([first_character]))

    while True:
        if buffer.remaining_bits < max_code_word_width:
            read_bytes = input_file.read(BYTES_TO_READ_AT_ONCE)
            buffer.add_bytes(read_bytes)
            if debug:
                print(f'Read {len(read_bytes)} bytes, current total: {buffer.total_bytes}', file=sys.stderr)
        if debug:
            print(f'Bits remaining in buffer: {buffer.remaining_bits}', file=sys.stderr)
        if buffer.remaining_bits < min_code_word_width:
            if debug:
                print(f'Exiting due to insufficient number of bits', file=sys.stderr)
            return  # EOF
        flag = bits_from_bytes(buffer, config.flag_width)
        if debug:
            print(f'Read flag {flag}, bits remaining in buffer: {buffer.remaining_bits}', file=sys.stderr)
        if is_literal(flag):
            literal = bits_from_bytes(buffer, BITS_IN_BYTE)
            if debug:
                print(f'Read literal {chr(literal)}, bits remaining in buffer: {buffer.remaining_bits}\n', file=sys.stderr)
                print_debug(debug_index, (literal, ), config, window)
                debug_index += 1
            window.insert(literal)
            output_file.write(bytes([literal]))
        else:
            distance = bits_from_bytes(buffer, config.distance_width)
            if debug:
                print(f'Read distance {distance}, bits remaining in buffer: {buffer.remaining_bits}', file=sys.stderr)
            length = bits_from_bytes(buffer, config.length_width)
            if debug:
                print(f'Read length {length}, bits remaining in buffer: {buffer.remaining_bits}\n', file=sys.stderr)
            length += config.length_bias
            if debug:
                print_debug(debug_index, (distance, length), config, window)
                debug_index += 1
            referenced_characters = window.at(distance, length)
            window.insert_multiple(referenced_characters)
            output_file.write(referenced_characters)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='LZSS sliding window decoder')
    parser.add_argument('input_file', type=argparse.FileType('rb'), nargs='?', help='Input file (to be decoded)')
    parser.add_argument('output_file', type=argparse.FileType('wb'), nargs='?', help='Output file (target)')
    parser.add_argument('--window-size', '-w', type=int, default=256, help='Sliding window size (in bytes)')
    parser.add_argument('--length-width', '-l', type=int, default=8, help='Reference length width (in bits)')
    parser.add_argument('--length-bias', '-b', type=int, default=0, help='Reference length bias')
    parser.add_argument('--distance-width', type=int, default=0, help='Reference distance width (in bits); zero means auto')
    parser.add_argument('--flag-width', type=int, default=1, help='Flag width (in bits)')
    parser.add_argument('--invert-flag', action='store_true', help='Treat zero as literal flag and others as reference flag')
    parser.add_argument('--back-distance', action='store_true', help='Count distance from the end of the window')
    parser.add_argument('--debug', action='store_true', help='Run in debug mode')
    args = parser.parse_args()

    input_file = args.input_file if args.input_file is not None else sys.stdin.buffer
    output_file = args.output_file if args.output_file is not None else sys.stdout.buffer
    # input_file = open(r'D:\Programowanie\studia\KODA\koda-lzss\py\examples\aaaaaaaaaaaaaaa.lzss', 'rb')
    # output_file = sys.stdout.buffer
    config = LzssConfig(args.window_size, args.length_width, args.length_bias, args.distance_width, args.flag_width, not args.invert_flag, args.back_distance)
    decode(input_file, output_file, config, args.debug)
 