import { describe, expect, it } from 'vitest';

import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { renderJsonReport } from '../../src/reporter/json-reporter.js';
import { createReportFromDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('fix strategy reporting', () => {
  it('renders recommended first fixes and strategy trade-offs', async () => {
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const json = JSON.parse(renderJsonReport(report));
    const html = renderHtmlReport(report);

    expect(
      json.summary.recommendedFirstFixes.map((fix: { code: string; priority: number }) => [
        fix.priority,
        fix.code
      ])
    ).toEqual([
      [1, 'E0382'],
      [2, 'E0499'],
      [3, 'E0502']
    ]);
    expect(html).toContain('<h2>Recommended First Fixes</h2>');
    expect(html).toContain('<h2>Fix Strategies</h2>');
    expect(html).toContain('Use clone() only when two owned values are intentional');
    expect(html).toContain(
      'Can hide a design issue if borrowing or moving later would express the intent better.'
    );
    expect(html).toContain('Shorten the first mutable borrow scope');
    expect(html).toContain('Move the mutation after the last shared read');
  });
});
