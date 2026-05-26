# cJSON Upstream

## Source

- Repository: `https://github.com/DaveGamble/cJSON.git`
- Selected tag: `v1.7.19`
- Selected commit: `c859b25da02955fef659d658b8f324b5cde87be3`
- License: MIT
- Local checkout path: `validation/ports/cjson/upstream/cjson/`

The local checkout is intentionally ignored by Git. This repository tracks the selected upstream version and acquisition commands, not a vendored copy of cJSON.

## Acquisition

```bash
git clone --depth 1 --branch v1.7.19 \
  https://github.com/DaveGamble/cJSON.git \
  validation/ports/cjson/upstream/cjson
```

Verify the selected version:

```bash
git -C validation/ports/cjson/upstream/cjson rev-parse HEAD
git -C validation/ports/cjson/upstream/cjson describe --tags --always
```

Expected output:

```text
c859b25da02955fef659d658b8f324b5cde87be3
v1.7.19
```

## Files Of Interest

- `cJSON.h`: public `cJSON` node structure, type flags, memory-management contract, and public parser functions
- `cJSON.c`: parser, printer, allocation hooks, linked-list node ownership, and recursive cleanup
- `tests/parse_value.c`: focused value parser tests
- `tests/parse_string.c`: focused string parser tests
- `tests/parse_array.c`: array parser behavior
- `tests/parse_object.c`: object parser behavior
- `tests/parse_examples.c`: end-to-end parser examples

## Ownership Signals

- `struct cJSON` stores `next`, `prev`, and `child` pointers for array/object chains.
- `cJSON_Delete` recursively frees child nodes unless `cJSON_IsReference` is set.
- `valuestring` and `string` have separate ownership flags.
- `parse_array` and `parse_object` allocate linked-list nodes and call `cJSON_Delete(head)` on partial parse failure.
- `parse_object` parses a key as `valuestring`, then transfers it into `string` by assigning `current_item->string = current_item->valuestring` and clearing `valuestring`.

These signals make cJSON a useful first validation target for ownership-error navigation during Rust migration.
