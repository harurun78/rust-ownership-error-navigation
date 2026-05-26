import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapE0502Diagnostic } from '../../src/mapper/e0502.js';
import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { createDiagnosticReport } from '../../src/reporter/json-reporter.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('borrow conflict HTML reporter', () => {
  it('renders causal event ordering before rustc span ordering for E0502', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapE0502Diagnostic(
      normalizeRustcDiagnostic(messages[2]!.message!, { diagnosticId: 'diagnostic-e0502' })
    );
    const html = renderHtmlReport(
      createDiagnosticReport({
        input: { path: 'ownership-baseline.jsonl' },
        diagnostics: [diagnostic]
      })
    );

    expect(html.indexOf('borrow_shared')).toBeLessThan(html.indexOf('borrow_mut'));
    expect(html.indexOf('borrow_mut')).toBeLessThan(
      html.indexOf('immutable borrow later used here')
    );
  });
});
