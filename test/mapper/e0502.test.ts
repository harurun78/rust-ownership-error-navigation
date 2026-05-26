import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0502Diagnostic } from '../../src/mapper/e0502.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0502 ownership mapper', () => {
  it('maps immutable borrow, mutable borrow conflict, and later immutable use context', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const record = mapE0502Diagnostic(
      normalizeRustcDiagnostic(messages[2]!.message!, { diagnosticId: 'diagnostic-e0502' })
    );

    expect(record.supported).toBe(true);
    expect(record.events?.map((event) => [event.kind, event.role, event.place])).toEqual([
      ['borrow_mut', 'conflict', 'v'],
      ['borrow_shared', 'cause', 'v'],
      ['context', 'context', 'v']
    ]);
    expect(record.spans.map((span) => span.role)).toEqual(['conflict', 'cause', 'context']);
    expect(record.events?.every((event) => event.evidence.length > 0)).toBe(true);
    expect(record.events?.every((event) => event.confidence === 'high')).toBe(true);
  });
});
