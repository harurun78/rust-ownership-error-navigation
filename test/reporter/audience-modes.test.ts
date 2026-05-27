import { describe, expect, it } from 'vitest';

import { createReportFromDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('audience mode report surfaces', () => {
  it('changes learner summary wording while preserving underlying events and spans', async () => {
    const beginner = await createReportFromDiagnosticFixture(
      'ownership-baseline-2026-05-24.jsonl',
      {
        audienceMode: 'beginner'
      }
    );
    const intermediate = await createReportFromDiagnosticFixture(
      'ownership-baseline-2026-05-24.jsonl',
      { audienceMode: 'intermediate' }
    );
    const agent = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'agent'
    });
    const beginnerDiagnostic = beginner.diagnostics.find(
      (diagnostic) => diagnostic.code === 'E0382'
    )!;
    const intermediateDiagnostic = intermediate.diagnostics.find(
      (diagnostic) => diagnostic.code === 'E0382'
    )!;
    const agentDiagnostic = agent.diagnostics.find((diagnostic) => diagnostic.code === 'E0382')!;

    expect(beginnerDiagnostic.learnerSummary).toMatchObject({
      audience: 'beginner',
      whatHappened: expect.stringContaining('was moved')
    });
    expect(intermediateDiagnostic.learnerSummary).toMatchObject({
      audience: 'intermediate',
      nextStep: expect.stringContaining('ownership boundary')
    });
    expect(agentDiagnostic.learnerSummary).toMatchObject({
      audience: 'agent',
      nextStep: expect.stringContaining('Repair hint:')
    });
    expect(intermediateDiagnostic.learnerSummary?.whatHappened).not.toBe(
      beginnerDiagnostic.learnerSummary?.whatHappened
    );
    for (const code of ['E0382', 'E0499', 'E0502']) {
      const beginnerRecord = beginner.diagnostics.find((diagnostic) => diagnostic.code === code)!;
      const intermediateRecord = intermediate.diagnostics.find(
        (diagnostic) => diagnostic.code === code
      )!;
      const agentRecord = agent.diagnostics.find((diagnostic) => diagnostic.code === code)!;

      expect(intermediateRecord.events).toEqual(beginnerRecord.events);
      expect(agentRecord.events).toEqual(beginnerRecord.events);
      expect(intermediateRecord.spans).toEqual(beginnerRecord.spans);
      expect(agentRecord.spans).toEqual(beginnerRecord.spans);
      expect(intermediateRecord.learnerSummary?.evidence).toEqual(
        beginnerRecord.learnerSummary?.evidence
      );
      expect(agentRecord.learnerSummary?.evidence).toEqual(beginnerRecord.learnerSummary?.evidence);
    }
  });
});
