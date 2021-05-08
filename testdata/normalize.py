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

            assert len(tokens) > 1
            opcode = tokens[0]
            op_a = tokens[1] + tokens[2] if len(tokens) > 2 else tokens[1]
            op_b = "".join(after).strip() or ""

            lines.append(f"{opcode:<8}{op_a+',':<8}{op_b}".strip())

        for i, line in enumerate(lines):
            if not line.startswith(";"):
                lines[i:i] = [f"{'ORG':<8}{org}"]
                break

        file_path.write_text("\n".join(lines))


if __name__ == "__main__":
    main()
