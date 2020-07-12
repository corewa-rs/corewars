# Label expansion and substitution

## `EQU` expansion

- For each line:
  - Parse with grammar
  - If first token is label:
    - If label is declared as substitution:
      - Expand + continue loop
    - Else:
      - Store offset for label
  - For remaining token in line:
    - If token is label:
      - If declared as Substitution:
        - Expand and continue loop
      - Else:
        - Save for later or ignore (shouldn't matter?)

## `FOR` expansion

TODO.

## Label substitution

- For each line:
  - Parse with grammar
  - For each token:
    - If token is label with value:
      - Substitute offset
    - Error for no value
