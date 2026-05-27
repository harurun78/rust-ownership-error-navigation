import { describe, expect, it } from 'vitest';

import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { renderJsonReport } from '../../src/reporter/json-reporter.js';
import { createReportFromDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('non-ownership diagnostic reporting', () => {
  it('groups non-ownership diagnostics in JSON summary and HTML', async () => {
    const report = await createReportFromDiagnosticFixture(
      'porting/non-ownership-navigation-2026-05-25.jsonl'
    );
    const json = JSON.parse(renderJsonReport(report));
    const html = renderHtmlReport(report);

    expect(json.summary).toMatchObject({
      totalDiagnostics: 3,
      supportedDiagnostics: 3,
      unsupportedDiagnostics: 0,
      ownershipDiagnostics: 0,
      nonOwnershipDiagnostics: 3
    });
    expect(
      json.diagnostics.map((diagnostic: { code: string; supported: boolean }) => [
        diagnostic.code,
        diagnostic.supported
      ])
    ).toEqual([
      ['E0308', true],
      ['E0004', true],
      ['E0425', true]
    ]);
    expect(html).toContain('<h2>Diagnostic Groups</h2>');
    expect(html).toContain('<td>Non-Ownership</td><td>3</td><td>E0308, E0004, E0425</td>');
    expect(html).toContain('<td>non-ownership</td><td>E0308</td>');
    expect(html).toContain('<td>non-ownership</td><td>E0004</td>');
    expect(html).toContain('<td>non-ownership</td><td>E0425</td>');
  });
});
