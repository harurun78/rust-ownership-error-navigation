import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import { main } from '../../src/cli/main.js';
import { diagnosticFixturePath } from '../helpers/diagnostic-fixtures.js';

let outputDirectory: string | undefined;

describe('E0382 report CLI integration', () => {
  afterEach(async () => {
    if (outputDirectory !== undefined) {
      await rm(outputDirectory, { recursive: true, force: true });
      outputDirectory = undefined;
    }
  });

  it('writes JSON and HTML reports for the baseline fixture', async () => {
    outputDirectory = await mkdtemp(join(tmpdir(), 'ownership-report-'));
    const jsonOut = join(outputDirectory, 'report.json');
    const htmlOut = join(outputDirectory, 'report.html');

    const exitCode = await main([
      '--input',
      diagnosticFixturePath('ownership-baseline-2026-05-24.jsonl'),
      '--json-out',
      jsonOut,
      '--html-out',
      htmlOut
    ]);

    expect(exitCode).toBe(0);

    const json = JSON.parse(await readFile(jsonOut, 'utf8'));
    const html = await readFile(htmlOut, 'utf8');

    expect(json.schemaVersion).toBe('0.1.0');
    expect(
      json.diagnostics.some((diagnostic: { code: string }) => diagnostic.code === 'E0382')
    ).toBe(true);
    expect(html).toContain('Rust Ownership Diagnostic Report');
    expect(html).toContain('borrow of moved value');
  });
});
