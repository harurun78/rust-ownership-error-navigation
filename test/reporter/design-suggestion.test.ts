import { describe, expect, it } from 'vitest';

import type { DiagnosticRecord } from '../../src/mapper/ownership-event.js';
import { renderHtmlReport } from '../../src/reporter/html-reporter.js';
import { createDiagnosticReport, renderJsonReport } from '../../src/reporter/json-reporter.js';
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

  it('renders design translation guidance for arena, stable IDs, and buffer spans', () => {
    const html = renderHtmlReport(
      createDiagnosticReport({
        input: { path: 'synthetic.jsonl' },
        diagnostics: [createDiagnosticWithDesignSuggestions()]
      })
    );

    expect(html).toContain('Design Translation');
    expect(html).toContain(
      'Direct object references to arena-owned nodes with parent and child IDs.'
    );
    expect(html).toContain(
      'Stored node references to stable NodeId values resolved through the arena.'
    );
    expect(html).toContain('Stored borrowed buffer views to spans or owned parse records.');
  });
});

function createDiagnosticWithDesignSuggestions(): DiagnosticRecord {
  return {
    id: 'diagnostic-design-translation',
    code: 'E0499',
    supported: true,
    level: 'error',
    message: 'cannot borrow `nodes` as mutable more than once at a time',
    spans: [],
    events: [],
    children: [],
    designSuggestions: [
      {
        id: 'arena-suggestion',
        diagnosticId: 'diagnostic-design-translation',
        kind: 'arena-backed-tree',
        title: 'Use arena-backed tree storage',
        why: 'Direct child references keep multiple mutable borrows open.',
        whenToUse: 'Use for DOM-like object graphs.',
        caution: 'Validate traversal semantics.',
        evidence: [{ source: 'heuristic', field: 'rule', value: 'arena-backed-tree' }],
        confidence: 'medium'
      },
      {
        id: 'node-id-suggestion',
        diagnosticId: 'diagnostic-design-translation',
        kind: 'stable-node-id',
        title: 'Use stable node IDs',
        why: 'Links need identity without long-lived references.',
        whenToUse: 'Use for parent and child relationships.',
        caution: 'Keep IDs validated at lookup boundaries.',
        evidence: [{ source: 'heuristic', field: 'rule', value: 'stable-node-id' }],
        confidence: 'medium'
      },
      {
        id: 'buffer-span-suggestion',
        diagnosticId: 'diagnostic-design-translation',
        kind: 'avoid-long-lived-buffer-borrow',
        title: 'Store spans instead of borrowed buffer views',
        why: 'Queued borrowed slices keep the buffer borrowed.',
        whenToUse: 'Use for streaming parsers.',
        caution: 'Resolve spans only while the buffer is stable.',
        evidence: [{ source: 'heuristic', field: 'rule', value: 'avoid-long-lived-buffer-borrow' }],
        confidence: 'medium'
      }
    ]
  };
}
