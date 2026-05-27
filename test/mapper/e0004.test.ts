import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0004Diagnostic } from '../../src/mapper/e0004.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0004 non-ownership mapper', () => {
  it('maps missing match coverage and an add-match-arm fix strategy', async () => {
    const messages = await loadDiagnosticFixture(
      'porting/non-ownership-navigation-2026-05-25.jsonl'
    );
    const message = messages.find((item) => item.message?.code?.code === 'E0004');
    const record = mapE0004Diagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0004' })
    );

    expect(record.supported).toBe(true);
    expect(record.unsupportedReason).toBeUndefined();
    expect(record.spans.map((span) => span.role)).toEqual(['conflict']);
    expect(record.children?.[0]?.spans.map((span) => span.role)).toEqual(['context', 'cause']);
    expect(record.children?.[1]?.spans[0]?.role).toBe('possible_fix');
    expect(record.fixStrategies?.[0]).toMatchObject({
      kind: 'add_match_arm',
      title: 'Cover every possible match case',
      spanId: 'diagnostic-e0004-child-2-span-1',
      confidence: 'high'
    });
  });
});
