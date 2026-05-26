import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostic } from '../../src/mapper/index.js';
import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { createDiagnosticReport, renderJsonReport } from '../../src/reporter/json-reporter.js';

describe('unsupported diagnostic reporting', () => {
  it('preserves unsupported diagnostic payload in JSON and HTML', () => {
    const diagnostic = mapDiagnostic(
      normalizeRustcDiagnostic(
        {
          code: { code: 'E0597' },
          level: 'error',
          message: 'borrowed value does not live long enough',
          rendered: 'error[E0597]: borrowed value does not live long enough',
          children: [],
          spans: []
        },
        { diagnosticId: 'unsupported-e0597' }
      )
    );
    const report = createDiagnosticReport({
      input: { path: 'unsupported.jsonl' },
      diagnostics: [diagnostic]
    });
    const json = JSON.parse(renderJsonReport(report));
    const html = renderHtmlReport(report);

    expect(json.diagnostics[0]).toMatchObject({
      code: 'E0597',
      supported: false,
      message: 'borrowed value does not live long enough',
      rendered: 'error[E0597]: borrowed value does not live long enough',
      unsupportedReason: 'Diagnostic code E0597 is outside the Phase 1 ownership mapping scope.'
    });
    expect(html).toContain('Unsupported Diagnostics');
    expect(html).toContain('Diagnostic code E0597 is outside the Phase 1 ownership mapping scope.');
  });
});
