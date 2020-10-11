#!/usr/bin/env python3
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
            tokens = line.strip().split()

            if not tokens:
                continue

            if tokens[0] == "ORG":
                continue

            if tokens[0] == "START":
                org = i - 1
                tokens.pop(0)

            opcode = tokens[0]
            op_a = tokens[1] + tokens[2]
            op_b = tokens[3] + tokens[4] if len(tokens) > 3 else ""

            lines.append(f"{opcode:<8}{op_a:<8}{op_b}")

        file_path.write_text("\n".join([f"ORG     {org}"] + lines))


if __name__ == "__main__":
    main()
