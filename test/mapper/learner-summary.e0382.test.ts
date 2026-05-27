import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostic } from '../../src/mapper/index.js';
import { createLearnerSummary } from '../../src/mapper/learner-summary.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('E0382 learner summary', () => {
  it('explains moved value, later use, and safest next action', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapDiagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0382' })
    );

    expect(diagnostic.learnerSummary).toMatchObject({
      audience: 'beginner',
      whatHappened: expect.stringContaining('s was moved'),
      whyItMatters: expect.stringContaining('no longer owns'),
      nextStep: expect.stringContaining('move location'),
      conceptTerms: ['move', 'ownership', 'use after move'],
      confidence: 'high'
    });
    expect(diagnostic.learnerSummary?.evidence.length).toBeGreaterThan(0);
  });

  it('can create the same summary as a pure helper for supported diagnostics', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapDiagnostic(
      normalizeRustcDiagnostic(message!.message!, { diagnosticId: 'diagnostic-e0382' })
    );

    expect(createLearnerSummary(diagnostic)).toEqual(diagnostic.learnerSummary);
  });
});
