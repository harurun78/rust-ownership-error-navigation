import { describe, expect, it } from 'vitest';

import { createRecommendedFirstFixes } from '../../src/mapper/recommended-first-fixes.js';
import type { DiagnosticRecord } from '../../src/mapper/ownership-event.js';

describe('recommended first fixes', () => {
  it('orders upstream non-ownership blockers before ownership diagnostics', () => {
    const fixes = createRecommendedFirstFixes([
      createDiagnostic('diagnostic-e0382', 'E0382'),
      createDiagnostic('diagnostic-e0308', 'E0308'),
      createDiagnostic('diagnostic-e0425', 'E0425'),
      createDiagnostic('diagnostic-e0004', 'E0004')
    ]);

    expect(fixes.map((fix) => [fix.priority, fix.code, fix.diagnosticId])).toEqual([
      [1, 'E0425', 'diagnostic-e0425'],
      [2, 'E0308', 'diagnostic-e0308'],
      [3, 'E0004', 'diagnostic-e0004'],
      [4, 'E0382', 'diagnostic-e0382']
    ]);
  });

  it('keeps input order stable for diagnostics with the same priority', () => {
    const fixes = createRecommendedFirstFixes([
      createDiagnostic('diagnostic-a', 'E0382'),
      createDiagnostic('diagnostic-b', 'E0382')
    ]);

    expect(fixes.map((fix) => fix.diagnosticId)).toEqual(['diagnostic-a', 'diagnostic-b']);
  });

  it('omits single diagnostics and unsupported records', () => {
    expect(createRecommendedFirstFixes([createDiagnostic('single', 'E0382')])).toEqual([]);
    expect(
      createRecommendedFirstFixes([
        createDiagnostic('unsupported', 'E0597', false),
        createDiagnostic('missing-code', null, false)
      ])
    ).toEqual([]);
  });
});

function createDiagnostic(id: string, code: string | null, supported = true): DiagnosticRecord {
  return {
    id,
    code,
    supported,
    message: id,
    spans: [],
    fixStrategies: supported
      ? [
          {
            id: `${id}-fix-1`,
            diagnosticId: id,
            kind: 'unknown',
            title: `Fix ${code ?? 'unknown'}`,
            rationale: 'test rationale',
            tradeOffs: ['test trade-off'],
            evidence: [{ source: 'diagnostic_code', field: 'code', value: code }],
            confidence: 'medium'
          }
        ]
      : undefined
  };
}
