import { describe, expect, it } from 'vitest';

import { renderJsonReport } from '../../src/reporter/json-reporter.js';
import { createReportFromDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('learner summary JSON reporter', () => {
  it('preserves learner summaries in rendered JSON without changing summary counts', async () => {
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'beginner'
    });
    const rendered = JSON.parse(renderJsonReport(report));

    expect(rendered.summary).toMatchObject({
      totalDiagnostics: 5,
      supportedDiagnostics: 3,
      unsupportedDiagnostics: 2
    });
    expect(rendered.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: 'E0382',
          learnerSummary: expect.objectContaining({
            audience: 'beginner',
            whatHappened: expect.stringContaining('was moved'),
            nextStep: expect.stringContaining('move location')
          }),
          events: expect.any(Array),
          spans: expect.any(Array)
        })
      ])
    );
  });

  it('does not duplicate source text or suggested code in learner summary evidence', async () => {
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'beginner'
    });
    const summaryEvidenceSources = report.diagnostics.flatMap(
      (diagnostic) => diagnostic.learnerSummary?.evidence.map((evidence) => evidence.source) ?? []
    );

    expect(summaryEvidenceSources).not.toContain('rustc_span_text');
    expect(summaryEvidenceSources).not.toContain('rustc_suggestion');
    expect(summaryEvidenceSources).toContain('diagnostic_code');
  });
});
