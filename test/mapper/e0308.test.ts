import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0308Diagnostic } from '../../src/mapper/e0308.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0308 non-ownership mapper', () => {
  it('maps type mismatch spans and an align-types fix strategy', async () => {
    const messages = await loadDiagnosticFixture(
      'porting/non-ownership-navigation-2026-05-25.jsonl'
    );
    const message = messages.find((item) => item.message?.code?.code === 'E0308');
    const record = mapE0308Diagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0308' })
    );

    expect(record.supported).toBe(true);
    expect(record.unsupportedReason).toBeUndefined();
    expect(record.spans.map((span) => span.role)).toEqual(['conflict', 'cause']);
    expect(record.children?.[0]?.spans[0]?.role).toBe('possible_fix');
    expect(record.fixStrategies?.[0]).toMatchObject({
      kind: 'align_types',
      title: 'Align the expression type with the expected type',
      spanId: 'diagnostic-e0308-child-1-span-1',
      confidence: 'high'
    });
  });
});
