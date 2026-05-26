import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { afterEach, describe, expect, it } from 'vitest';

import { main } from '../../src/cli/main.js';
import { diagnosticFixturePath } from '../helpers/diagnostic-fixtures.js';

let outputDirectory: string | undefined;

describe('quickstart workflow', () => {
  afterEach(async () => {
    if (outputDirectory !== undefined) {
      await rm(outputDirectory, { recursive: true, force: true });
      outputDirectory = undefined;
    }
  });

  it('creates requested JSON and HTML outputs without mutating other files', async () => {
    outputDirectory = await mkdtemp(join(tmpdir(), 'quickstart-report-'));
    const jsonOut = join(outputDirectory, 'ownership-report.json');
    const htmlOut = join(outputDirectory, 'ownership-report.html');

    const exitCode = await main([
      '--input',
      diagnosticFixturePath('ownership-baseline-2026-05-24.jsonl'),
      '--json-out',
      jsonOut,
      '--html-out',
      htmlOut
    ]);

    expect(exitCode).toBe(0);
    expect((await readdir(outputDirectory)).sort()).toEqual([
      'ownership-report.html',
      'ownership-report.json'
    ]);

    const schema = JSON.parse(
      await readFile(
        'specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json',
        'utf8'
      )
    );
    const json = JSON.parse(await readFile(jsonOut, 'utf8'));
    const html = await readFile(htmlOut, 'utf8');
    const validate = new Ajv2020({ allErrors: true, strict: false }).compile(schema);

    expect(validate(json), JSON.stringify(validate.errors, null, 2)).toBe(true);
    expect(html).toContain('Borrow Sheet');
    expect(html).toContain('Unsupported Diagnostics');
  });
});
