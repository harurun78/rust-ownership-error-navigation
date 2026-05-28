import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { attachDesignSuggestions } from '../../src/mapper/design-suggestion.js';
import { mapE0308Diagnostic } from '../../src/mapper/e0308.js';
import { mapE0502Diagnostic } from '../../src/mapper/e0502.js';
import { mapDiagnostics } from '../../src/mapper/index.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('deterministic design suggestions', () => {
  it('adds split-mutation-phase to borrow conflicts with cause, conflict, and context', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostics = mapDiagnostics(
      messages.map((message, index) =>
        normalizeRustcDiagnostic(message.message!, { diagnosticIndex: index })
      ),
      undefined,
      { audienceMode: 'intermediate' }
    );
    const diagnostic = diagnostics.find((record) => record.code === 'E0502')!;

    expect(diagnostic.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'split-mutation-phase',
          confidence: 'high',
          evidence: expect.arrayContaining([
            expect.objectContaining({ source: 'heuristic', field: 'rule' })
          ])
        })
      ])
    );
  });

  it('adds avoid-long-lived-buffer-borrow when borrow conflict evidence mentions parser buffers', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const mapped = mapE0502Diagnostic(
      normalizeRustcDiagnostic(messages[2]!.message!, { diagnosticId: 'diagnostic-e0502-buffer' })
    );
    const diagnostic = attachDesignSuggestions({
      ...mapped,
      message: `${mapped.message} in parser output buffer`,
      spans: mapped.spans.map((span) => ({
        ...span,
        snippet: span.snippet === undefined ? 'stream.next_out buffer' : span.snippet
      }))
    });

    expect(diagnostic.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'avoid-long-lived-buffer-borrow',
          confidence: 'medium'
        })
      ])
    );
  });

  it('adds owned-result for moved value reuse and E0308 type boundary pressure', async () => {
    const ownershipMessages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const ownershipDiagnostics = mapDiagnostics(
      ownershipMessages.map((message, index) =>
        normalizeRustcDiagnostic(message.message!, { diagnosticIndex: index })
      )
    );
    const e0382 = ownershipDiagnostics.find((record) => record.code === 'E0382')!;
    const portingMessages = await loadDiagnosticFixture(
      'porting/non-ownership-navigation-2026-05-25.jsonl'
    );
    const e0308Message = portingMessages.find((message) => message.message?.code?.code === 'E0308');
    const e0308 = attachDesignSuggestions(
      mapE0308Diagnostic(
        normalizeRustcDiagnostic(e0308Message!.message!, { diagnosticId: 'diagnostic-e0308' })
      )
    );

    expect(e0382.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: 'owned-result', confidence: 'high' })
      ])
    );
    expect(e0308.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: 'owned-result', confidence: 'medium' })
      ])
    );
  });
});
