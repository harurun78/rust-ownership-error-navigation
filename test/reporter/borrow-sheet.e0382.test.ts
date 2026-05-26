import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0382Diagnostic } from '../../src/mapper/e0382.js';
import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { createDiagnosticReport } from '../../src/reporter/json-reporter.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0382 Borrow Sheet HTML', () => {
  it('renders Borrow Sheet alongside causality view for E0382 events', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapE0382Diagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0382' })
    );
    const html = renderHtmlReport(
      createDiagnosticReport({
        input: { path: 'ownership-baseline.jsonl' },
        diagnostics: [diagnostic]
      })
    );

    expect(html).toContain('Causality Timeline');
    expect(html).toContain('Borrow Sheet');
    expect(html).toContain('move');
    expect(html).toContain('use');
    expect(html).toContain('context');
    expect(html).toContain('possible_fix');
  });
});
