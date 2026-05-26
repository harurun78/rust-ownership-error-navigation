import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0382Diagnostic } from '../../src/mapper/e0382.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0382 ownership mapper', () => {
  it('maps move cause, use conflict, context, and possible fix events', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const record = mapE0382Diagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0382' })
    );

    expect(record.supported).toBe(true);
    expect(record.events?.map((event) => [event.kind, event.role, event.place])).toEqual([
      ['move', 'cause', 's'],
      ['use', 'conflict', 's'],
      ['context', 'context', 's'],
      ['possible_fix', 'possible_fix', 's']
    ]);
    expect(record.spans.map((span) => span.role)).toEqual(['cause', 'conflict', 'context']);
    expect(record.events?.every((event) => event.evidence.length > 0)).toBe(true);
    expect(record.events?.every((event) => event.confidence === 'high')).toBe(true);
  });
});
