import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import { main } from '../../src/cli/main.js';
import { diagnosticFixturePath } from '../helpers/diagnostic-fixtures.js';

let outputDirectory: string | undefined;

describe('borrow conflict CLI integration', () => {
  afterEach(async () => {
    if (outputDirectory !== undefined) {
      await rm(outputDirectory, { recursive: true, force: true });
      outputDirectory = undefined;
    }
  });

  it('writes E0499 and E0502 JSON and HTML events from the baseline fixture', async () => {
    outputDirectory = await mkdtemp(join(tmpdir(), 'borrow-conflicts-'));
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

    expect(json.summary).toMatchObject({
      totalDiagnostics: 5,
      supportedDiagnostics: 3,
      unsupportedDiagnostics: 2
    });
    expect(
      json.diagnostics.find((diagnostic: { code: string }) => diagnostic.code === 'E0499').events
    ).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: 'borrow_mut_request', role: 'conflict' })
      ])
    );
    expect(
      json.diagnostics.find((diagnostic: { code: string }) => diagnostic.code === 'E0502').events
    ).toEqual(
      expect.arrayContaining([expect.objectContaining({ kind: 'borrow_shared', role: 'cause' })])
    );
    expect(html).toContain('first mutable borrow occurs here');
    expect(html).toContain('immutable borrow occurs here');
  });
});
