import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0382Diagnostic } from '../../src/mapper/e0382.js';
import { createDiagnosticReport } from '../../src/reporter/json-reporter.js';
import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0382 HTML reporter', () => {
  it('renders supported diagnostic sections and possible fix rows', async () => {
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

    expect(html).toContain('Summary');
    expect(html).toContain('Causality Timeline');
    expect(html).toContain('Source Spans');
    expect(html).toContain('Evidence');
    expect(html).toContain('possible_fix');
    expect(html).toContain('value borrowed here after move');
    expect(html).toContain('println!(&quot;{}&quot;, s);');
  });
});
