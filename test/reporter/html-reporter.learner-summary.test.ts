import { describe, expect, it } from 'vitest';

import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { createReportFromDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('learner summary HTML reporter', () => {
  it('renders learner summary cards before detailed diagnostic sections', async () => {
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'beginner'
    });
    const html = renderHtmlReport(report);

    expect(html).toContain('Learner Summaries');
    expect(html).toContain('E0382 learner summary');
    expect(html).toContain('What Happened');
    expect(html).toContain('s was moved, and the code later tried to use it again.');
    expect(html).toContain('Borrow Sheet');
    expect(html.indexOf('id="learner-summaries"')).toBeLessThan(html.indexOf('id="diagnostics"'));
    expect(html.indexOf('id="learner-summaries"')).toBeLessThan(
      html.indexOf('id="causality-timeline"')
    );
  });

  it('does not create summary cards for unsupported diagnostics', async () => {
    const report = await createReportFromDiagnosticFixture(
      'rustc-non-ownership-smoke-2026-05-25.jsonl'
    );
    const html = renderHtmlReport(report);

    expect(report.diagnostics.every((diagnostic) => diagnostic.learnerSummary === undefined)).toBe(
      true
    );
    expect(html).toContain('No learner summaries');
  });
});
