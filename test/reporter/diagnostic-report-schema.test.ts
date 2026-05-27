import { readFile } from 'node:fs/promises';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostics } from '../../src/mapper/index.js';
import { parseCargoMessagesFile } from '../../src/parser/cargo-message-parser.js';
import { createDiagnosticReport } from '../../src/reporter/json-reporter.js';
import {
  createReportFromDiagnosticFixture,
  diagnosticFixturePath
} from '../helpers/diagnostic-fixtures.js';

describe('diagnostic report JSON schema', () => {
  it('validates a generated baseline report against the contract', async () => {
    const schema = JSON.parse(
      await readFile(
        'specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json',
        'utf8'
      )
    );
    const cargoMessages = await parseCargoMessagesFile(
      diagnosticFixturePath('ownership-baseline-2026-05-24.jsonl')
    );
    const diagnostics = mapDiagnostics(
      cargoMessages.map((message, index) =>
        normalizeRustcDiagnostic(message.message!, { diagnosticIndex: index })
      )
    );
    const report = createDiagnosticReport({
      input: { path: 'ownership-baseline.jsonl' },
      diagnostics
    });
    const ajv = new Ajv2020({ allErrors: true, strict: false });
    const validate = ajv.compile(schema);

    expect(validate(report), JSON.stringify(validate.errors, null, 2)).toBe(true);
  });

  it('validates optional learner-centered report fields against the contract', async () => {
    const schema = JSON.parse(
      await readFile(
        'specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json',
        'utf8'
      )
    );
    const report = await createReportFromDiagnosticFixture('ownership-baseline-2026-05-24.jsonl', {
      audienceMode: 'beginner'
    });
    const diagnostic = report.diagnostics.find((record) => record.supported)!;
    const evidence = [{ source: 'heuristic' as const, field: 'test' }];

    diagnostic.learnerSummary = {
      audience: 'beginner',
      whatHappened: 'A value was moved and then used again.',
      whyItMatters: 'The original binding no longer owns the value after the move.',
      nextStep: 'Inspect the move span before changing ownership.',
      conceptTerms: ['move', 'ownership'],
      evidence,
      confidence: 'medium'
    };
    diagnostic.fixStrategies = [
      {
        id: 'fix-1',
        diagnosticId: diagnostic.id,
        kind: 'borrow',
        title: 'Borrow instead of moving',
        rationale: 'Borrowing can keep ownership with the original binding.',
        tradeOffs: ['Requires the callee to accept a reference.'],
        evidence,
        confidence: 'medium'
      }
    ];
    report.summary.recommendedFirstFixes = [
      {
        diagnosticId: diagnostic.id,
        code: diagnostic.code,
        priority: 1,
        reason: 'Supported ownership diagnostic with direct evidence.',
        nextStep: 'Read the learner summary first.',
        evidence,
        confidence: 'medium'
      }
    ];

    const ajv = new Ajv2020({ allErrors: true, strict: false });
    const validate = ajv.compile(schema);

    expect(validate(report), JSON.stringify(validate.errors, null, 2)).toBe(true);
  });
});
