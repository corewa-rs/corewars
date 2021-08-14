#!/usr/bin/env python3

"""
Simple helper script to make pMars load file output consistent for testing.
"""

import pathlib
import sys


def main():
    for arg in sys.argv[1:]:
        file_path = pathlib.Path(arg)

        if not file_path.exists():
            print(f"Error, file does not exist: {file_path}", file=sys.stderr)
            continue

        lines = []
        org = 0

        for i, line in enumerate(file_path.read_text().splitlines()):
            if line.startswith(";"):
                lines.append(line)
                continue

            before, *after = line.strip().split(",")
            tokens = before.split()

            if not tokens:
                continue

            if tokens[0] == "ORG":
                assert len(tokens) > 1
                org = tokens[1]
                continue

            if tokens[0] == "START":
                if org == "START":
                    org = i - 1

                tokens = tokens[1:]

            assert len(tokens) > 1
            opcode = tokens[0]
            op_a = tokens[1] + tokens[2] if len(tokens) > 2 else tokens[1]
            op_b = "".join(after).strip() or ""

            op_a = _normalize_operand(op_a)
            op_b = _normalize_operand(op_b)

            lines.append(f"{opcode:<8}{op_a + ',':<8}{op_b}".strip())

        for i, line in enumerate(lines):
            if not line.startswith(";"):
                lines[i:i] = [f"{'ORG':<8}{org}"]
                break

        file_path.write_text("\n".join(lines) + "\n")


def _normalize_operand(op: str) -> str:
    if op[0].isdigit():
        return op

    value = int(op[1:]) % 8000
    return f"{op[0]}{value}"


if __name__ == "__main__":
    main()
