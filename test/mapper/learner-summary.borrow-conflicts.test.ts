import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostic } from '../../src/mapper/index.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('borrow conflict learner summaries', () => {
  it('explains E0499 first mutable borrow and conflicting mutable borrow', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapDiagnostic(
      normalizeRustcDiagnostic(messages[1]!.message!, { diagnosticId: 'diagnostic-e0499' })
    );

    expect(diagnostic.learnerSummary).toMatchObject({
      audience: 'beginner',
      whatHappened: expect.stringContaining('another mutable borrow'),
      whyItMatters: expect.stringContaining('only one active mutable borrow'),
      nextStep: expect.stringContaining('first mutable borrow'),
      conceptTerms: ['mutable borrow', 'borrow scope', 'conflict'],
      confidence: 'high'
    });
    expect(diagnostic.learnerSummary?.evidence.length).toBeGreaterThan(0);
  });

  it('explains E0502 immutable borrow and conflicting mutable borrow', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = mapDiagnostic(
      normalizeRustcDiagnostic(messages[2]!.message!, { diagnosticId: 'diagnostic-e0502' })
    );

    expect(diagnostic.learnerSummary).toMatchObject({
      audience: 'beginner',
      whatHappened: expect.stringContaining('immutably borrowed'),
      whyItMatters: expect.stringContaining('shared reads and mutable writes'),
      nextStep: expect.stringContaining('last use of the immutable borrow'),
      conceptTerms: ['immutable borrow', 'mutable borrow', 'borrow scope'],
      confidence: 'high'
    });
    expect(diagnostic.learnerSummary?.evidence.length).toBeGreaterThan(0);
  });
});
