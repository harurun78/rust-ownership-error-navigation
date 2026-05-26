import { readFile } from 'node:fs/promises';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostics } from '../../src/mapper/index.js';
import { parseCargoMessagesFile } from '../../src/parser/cargo-message-parser.js';
import { createDiagnosticReport } from '../../src/reporter/json-reporter.js';
import { diagnosticFixturePath } from '../helpers/diagnostic-fixtures.js';

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
});
