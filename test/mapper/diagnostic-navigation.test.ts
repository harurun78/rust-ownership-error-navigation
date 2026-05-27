import { describe, expect, it } from 'vitest';

import type {
  AudienceMode,
  FixStrategy,
  LearnerSummary,
  RecommendedFirstFix
} from '../../src/mapper/diagnostic-navigation.js';

const evidence = [{ source: 'heuristic' as const, field: 'test' }];

describe('diagnostic navigation shared model', () => {
  it('represents learner summaries with audience, evidence, and confidence', () => {
    const summary = {
      audience: 'beginner',
      whatHappened: 'The value was moved before this later use.',
      whyItMatters: 'Rust prevents use-after-move because the original binding no longer owns it.',
      nextStep: 'Check the move span and decide whether borrowing is enough.',
      conceptTerms: ['move', 'ownership'],
      evidence,
      confidence: 'high'
    } satisfies LearnerSummary;

    expect(summary).toMatchObject({
      audience: 'beginner',
      confidence: 'high',
      conceptTerms: ['move', 'ownership']
    });
  });

  it('represents fix strategies with trade-offs', () => {
    const strategy = {
      id: 'fix-1',
      diagnosticId: 'diagnostic-1',
      kind: 'clone_or_copy',
      title: 'Clone before moving',
      rationale: 'Keep an owned value for later use.',
      tradeOffs: ['May allocate or duplicate data.', 'Can hide a better ownership boundary.'],
      spanId: 'span-1',
      evidence,
      confidence: 'medium'
    } satisfies FixStrategy;

    expect(strategy.kind).toBe('clone_or_copy');
    expect(strategy.tradeOffs).toHaveLength(2);
  });

  it('represents deterministic first-fix recommendations', () => {
    const recommendation = {
      diagnosticId: 'diagnostic-1',
      code: 'E0308',
      priority: 1,
      reason: 'Type mismatches often cause downstream diagnostics.',
      nextStep: 'Align the expression type with the expected type first.',
      evidence,
      confidence: 'medium'
    } satisfies RecommendedFirstFix;

    expect(recommendation).toMatchObject({ code: 'E0308', priority: 1 });
  });

  it('limits audience modes to learner and agent report surfaces', () => {
    const modes: AudienceMode[] = ['beginner', 'intermediate', 'agent'];

    expect(modes).toEqual(['beginner', 'intermediate', 'agent']);
  });
});
