import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0499Diagnostic } from '../../src/mapper/e0499.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0499 ownership mapper', () => {
  it('maps first mutable borrow, conflicting borrow, and later use context', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const record = mapE0499Diagnostic(
      normalizeRustcDiagnostic(messages[1]!.message!, { diagnosticId: 'diagnostic-e0499' })
    );

    expect(record.supported).toBe(true);
    expect(record.events?.map((event) => [event.kind, event.role, event.place])).toEqual([
      ['borrow_mut', 'cause', 'x'],
      ['borrow_mut_request', 'conflict', 'x'],
      ['context', 'context', 'x']
    ]);
    expect(record.spans.map((span) => span.role)).toEqual(['cause', 'conflict', 'context']);
    expect(record.events?.every((event) => event.evidence.length > 0)).toBe(true);
    expect(record.events?.every((event) => event.confidence === 'high')).toBe(true);
  });
});
