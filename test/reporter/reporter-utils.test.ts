import { describe, expect, it } from 'vitest';

import {
  createDiagnosticReportSummary,
  escapeHtml,
  stableId
} from '../../src/reporter/reporter-utils.js';
import type { DiagnosticRecord } from '../../src/mapper/ownership-event.js';

describe('reporter utilities', () => {
  it('escapes special HTML characters', () => {
    expect(escapeHtml(`<span data-x="1">Tom & 'Rust'</span>`)).toBe(
      '&lt;span data-x=&quot;1&quot;&gt;Tom &amp; &#39;Rust&#39;&lt;/span&gt;'
    );
  });

  it('generates deterministic stable ids with sanitized prefixes', () => {
    const first = stableId('Diagnostic Span', ['E0382', 'src/main.rs', 7, 21]);
    const second = stableId('Diagnostic Span', ['E0382', 'src/main.rs', 7, 21]);
    const other = stableId('Diagnostic Span', ['E0382', 'src/main.rs', 8, 20]);

    expect(first).toBe(second);
    expect(first).toMatch(/^diagnostic-span-[a-f0-9]{12}$/);
    expect(first).not.toBe(other);
  });

  it('creates diagnostic summary counts for supported and unsupported records', () => {
    const diagnostics = [
      createDiagnosticRecord('diagnostic-1', true),
      createDiagnosticRecord('diagnostic-2', false),
      createDiagnosticRecord('diagnostic-3', true)
    ];

    expect(createDiagnosticReportSummary(diagnostics)).toEqual({
      totalDiagnostics: 3,
      supportedDiagnostics: 2,
      unsupportedDiagnostics: 1,
      ownershipDiagnostics: 2,
      nonOwnershipDiagnostics: 0
    });
  });
});

function createDiagnosticRecord(id: string, supported: boolean): DiagnosticRecord {
  return {
    id,
    code: supported ? 'E0382' : null,
    supported,
    message: id,
    spans: []
  };
}
