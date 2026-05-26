# cJSON Upstream Analysis

## Snapshot

- Version: `v1.7.19`
- Commit: `c859b25da02955fef659d658b8f324b5cde87be3`
- Core files: `cJSON.c` and `cJSON.h`
- Approximate size: `cJSON.c` 3191 lines, `cJSON.h` 306 lines

## Important Ownership Shapes

`cJSON` nodes form linked chains with optional child lists:

- `next` / `prev`: sibling traversal for array/object entries
- `child`: nested array/object contents
- `valuestring`: string value storage
- `string`: object key storage

`cJSON_Delete` owns recursive cleanup. It walks siblings through `next`, recursively deletes `child`, frees `valuestring` unless the node is a reference, frees `string` unless the key is const, then frees the node.

`parse_array` and `parse_object` allocate nodes incrementally. On parse failure they call `cJSON_Delete(head)`, so partial construction is cleaned up by a single owner.

`parse_object` temporarily parses an object key into `valuestring`, then transfers ownership to `string` and clears `valuestring`. In Rust, the first pass should avoid that temporary dual-field state by parsing keys directly into owned `String` values.

## Initial Rust Design Choice

Use an owned `JsonValue` enum with `Vec<JsonValue>` arrays and `Vec<(String, JsonValue)>` objects. This deliberately removes C sibling pointers and cleanup flags so the first experiment focuses on parser ownership, recursive data, and mutation during construction.

Later experiments can reintroduce harder constraints such as borrowed string storage, custom allocation, arena allocation, or API compatibility.
