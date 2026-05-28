import { describe, expect, it } from 'vitest';

import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { renderJsonReport } from '../../src/reporter/json-reporter.js';
import { createReportFromDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('design suggestion reporters', () => {
  it('preserves design suggestions in rendered JSON', async () => {
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'agent'
    });
    const rendered = JSON.parse(renderJsonReport(report));

    expect(rendered.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: 'E0502',
          designSuggestions: expect.arrayContaining([
            expect.objectContaining({
              kind: 'split-mutation-phase',
              evidence: expect.any(Array)
            })
          ])
        })
      ])
    );
  });

  it('renders Design Direction before detailed causality sections', async () => {
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'intermediate'
    });
    const html = renderHtmlReport(report);

    expect(html).toContain('Design Direction');
    expect(html).toContain('split-mutation-phase');
    expect(html).toContain('owned-result');
    expect(html.indexOf('id="design-direction"')).toBeLessThan(
      html.indexOf('id="causality-timeline"')
    );
  });
});
