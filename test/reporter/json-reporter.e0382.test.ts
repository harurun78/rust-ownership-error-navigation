import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0382Diagnostic } from '../../src/mapper/e0382.js';
import { createDiagnosticReport, renderJsonReport } from '../../src/reporter/json-reporter.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0382 JSON reporter', () => {
  it('renders a schema-versioned report with E0382 events', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapE0382Diagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0382' })
    );
    const report = createDiagnosticReport({
      input: { path: 'test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl' },
      diagnostics: [diagnostic]
    });

    expect(report.summary).toEqual({
      totalDiagnostics: 1,
      supportedDiagnostics: 1,
      unsupportedDiagnostics: 0,
      ownershipDiagnostics: 1,
      nonOwnershipDiagnostics: 0
    });
    expect(JSON.parse(renderJsonReport(report))).toMatchObject({
      schemaVersion: '0.1.0',
      diagnostics: [
        {
          code: 'E0382',
          supported: true,
          events: [
            { kind: 'move', role: 'cause', spanId: 'diagnostic-e0382-span-1' },
            { kind: 'use', role: 'conflict', spanId: 'diagnostic-e0382-span-2' },
            { kind: 'context', role: 'context', spanId: 'diagnostic-e0382-span-3' },
            {
              kind: 'possible_fix',
              role: 'possible_fix',
              spanId: 'diagnostic-e0382-child-2-span-1'
            }
          ]
        }
      ]
    });
  });
});
