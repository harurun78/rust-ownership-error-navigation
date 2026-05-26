import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { parseCargoMessagesJsonl } from '../../src/parser/cargo-message-parser.js';

describe('parser and normalizer edge cases', () => {
  it('preserves null codes, empty children, null labels, missing primary spans, expansions, and mixed paths', () => {
    const messages = parseCargoMessagesJsonl(
      JSON.stringify({
        reason: 'compiler-message',
        message: {
          code: null,
          level: 'warning',
          message: 'edge case diagnostic',
          rendered: 'warning: edge case diagnostic',
          children: [],
          spans: [
            {
              file_name: 'src/main.rs',
              byte_start: 0,
              byte_end: 3,
              line_start: 1,
              line_end: 1,
              column_start: 1,
              column_end: 4,
              is_primary: false,
              label: null,
              text: [{ text: 'let x = 1;', highlight_start: 1, highlight_end: 4 }],
              suggested_replacement: null,
              suggestion_applicability: null,
              expansion: { macro_decl_name: 'println!' }
            },
            {
              file_name: 'C:\\project\\src\\lib.rs',
              byte_start: 10,
              byte_end: 12,
              line_start: 2,
              line_end: 2,
              column_start: 5,
              column_end: 7,
              is_primary: false,
              label: 'windows path span',
              text: [],
              suggested_replacement: null,
              suggestion_applicability: null,
              expansion: null
            }
          ]
        }
      })
    );

    const record = normalizeRustcDiagnostic(messages[0]!.message!, { diagnosticId: 'edge' });

    expect(record.code).toBeNull();
    expect(record.children).toEqual([]);
    expect(record.spans[0]).toMatchObject({ label: null, isPrimary: false, hasExpansion: true });
    expect(record.spans[1]).toMatchObject({ file: 'C:\\project\\src\\lib.rs' });
  });
});
