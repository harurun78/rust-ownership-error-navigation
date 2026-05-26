import { describe, expect, it } from 'vitest';

import {
  isSupportedDiagnosticCode,
  mapDiagnostic,
  mapDiagnostics
} from '../../src/mapper/index.js';
import type { DiagnosticRecord } from '../../src/mapper/ownership-event.js';

describe('ownership mapper registry', () => {
  it('identifies Phase 1 supported diagnostic codes', () => {
    expect(isSupportedDiagnosticCode('E0382')).toBe(true);
    expect(isSupportedDiagnosticCode('E0499')).toBe(true);
    expect(isSupportedDiagnosticCode('E0502')).toBe(true);
    expect(isSupportedDiagnosticCode('E0597')).toBe(false);
    expect(isSupportedDiagnosticCode(null)).toBe(false);
  });

  it('dispatches supported diagnostics to registered mapper entry points', () => {
    const diagnostic = createDiagnosticRecord('E0382');
    const mapped = mapDiagnostic(diagnostic, {
      E0382: (record) => ({ ...record, supported: true, message: 'mapped E0382' }),
      E0499: (record) => record,
      E0502: (record) => record
    });

    expect(mapped).toMatchObject({
      code: 'E0382',
      supported: true,
      message: 'mapped E0382'
    });
  });

  it('dispatches unsupported and missing codes to fallback records', () => {
    const mapped = mapDiagnostics([createDiagnosticRecord('E0597'), createDiagnosticRecord(null)]);

    expect(mapped).toEqual([
      expect.objectContaining({
        code: 'E0597',
        supported: false,
        unsupportedReason: 'Diagnostic code E0597 is outside the Phase 1 ownership mapping scope.'
      }),
      expect.objectContaining({
        code: null,
        supported: false,
        unsupportedReason: 'Diagnostic does not include a rustc error code.'
      })
    ]);
  });
});

function createDiagnosticRecord(code: string | null): DiagnosticRecord {
  return {
    id: code ?? 'null-code',
    code,
    supported: false,
    message: code ?? 'missing code',
    spans: []
  };
}
