import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0425Diagnostic } from '../../src/mapper/e0425.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0425 non-ownership mapper', () => {
  it('maps unresolved names and a resolve-name fix strategy', async () => {
    const messages = await loadDiagnosticFixture(
      'porting/non-ownership-navigation-2026-05-25.jsonl'
    );
    const message = messages.find((item) => item.message?.code?.code === 'E0425');
    const record = mapE0425Diagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0425' })
    );

    expect(record.supported).toBe(true);
    expect(record.unsupportedReason).toBeUndefined();
    expect(record.spans.map((span) => span.role)).toEqual(['conflict']);
    expect(record.fixStrategies?.[0]).toMatchObject({
      kind: 'resolve_name',
      title: 'Resolve the missing name in this scope',
      spanId: 'diagnostic-e0425-span-1',
      confidence: 'high'
    });
  });
});
