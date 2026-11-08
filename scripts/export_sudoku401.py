#!/usr/bin/env python3
"""Export the authorized Sudoku 401 SQLite board table to the compact Web asset.

Usage: python3 scripts/export_sudoku401.py /path/to/sudoku2.db assets/sudoku401.pack
The DB's solution column contains only the missing cells, not the full answer.
"""
import sqlite3
import struct
import sys
from collections import Counter
from pathlib import Path

MAGIC = b'S401PK1\0'

def main(source, output):
    db = sqlite3.connect(f'file:{Path(source).resolve()}?mode=ro', uri=True)
    records = db.execute('SELECT id, level, board, solution FROM board ORDER BY id').fetchall()
    counts, ids, packed = Counter(), set(), bytearray(MAGIC)
    for board_id, level, board, missing in records:
        if board_id in ids or not (0 < board_id < 2**32) or level not in range(1, 9):
            raise ValueError(f'invalid ID or level: {board_id}')
        ids.add(board_id)
        if len(board) != 81 or len(missing) != 81 or any(c not in '0123456789' for c in board + missing):
            raise ValueError(f'invalid digits: {board_id}')
        if any((a == '0') == (b == '0') for a, b in zip(board, missing)):
            raise ValueError(f'given/missing masks overlap or leave holes: {board_id}')
        full = ''.join(a if a != '0' else b for a, b in zip(board, missing))
        units = [[9*r+c for c in range(9)] for r in range(9)]
        units += [[9*r+c for r in range(9)] for c in range(9)]
        units += [[(b//3*3+r)*9+b%3*3+c for r in range(3) for c in range(3)] for b in range(9)]
        if any(set(full[i] for i in unit) != set('123456789') for unit in units):
            raise ValueError(f'invalid solution: {board_id}')
        packed.extend(struct.pack('<IB', board_id, level))
        packed.extend((int(a) << 4) | int(b) for a, b in zip(board, missing))
        counts[level] += 1
    if len(records) != 8987 or set(counts) != set(range(1, 9)):
        raise ValueError(f'unexpected dataset: {len(records)}, {counts}')
    Path(output).parent.mkdir(parents=True, exist_ok=True)
    Path(output).write_bytes(packed)
    print(f'{len(records)} validated puzzles; levels {dict(sorted(counts.items()))}; {len(packed)} bytes')

if __name__ == '__main__':
    if len(sys.argv) != 3: raise SystemExit(__doc__)
    main(*sys.argv[1:])
