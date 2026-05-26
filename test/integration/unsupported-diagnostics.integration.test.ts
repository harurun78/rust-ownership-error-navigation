import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import { main } from '../../src/cli/main.js';
import { diagnosticFixturePath } from '../helpers/diagnostic-fixtures.js';

let outputDirectory: string | undefined;

describe('unsupported diagnostics CLI compatibility', () => {
  afterEach(async () => {
    if (outputDirectory !== undefined) {
      await rm(outputDirectory, { recursive: true, force: true });
      outputDirectory = undefined;
    }
  });

  it.each([
    ['ownership-followup-2026-05-25.jsonl', 12, 0, 12],
    ['ownership-advanced-2026-05-25.jsonl', 10, 5, 5],
    ['rustc-non-ownership-smoke-2026-05-25.jsonl', 8, 0, 8]
  ])('retains unsupported diagnostics from %s', async (fixture, total, supported, unsupported) => {
    outputDirectory = await mkdtemp(join(tmpdir(), 'unsupported-diagnostics-'));
    const jsonOut = join(outputDirectory, `${fixture}.json`);
    const htmlOut = join(outputDirectory, `${fixture}.html`);

    const exitCode = await main([
      '--input',
      diagnosticFixturePath(fixture),
      '--json-out',
      jsonOut,
      '--html-out',
      htmlOut
    ]);

    expect(exitCode).toBe(0);

    const json = JSON.parse(await readFile(jsonOut, 'utf8'));
    const html = await readFile(htmlOut, 'utf8');

    expect(json.summary).toMatchObject({
      totalDiagnostics: total,
      supportedDiagnostics: supported,
      unsupportedDiagnostics: unsupported
    });
    expect(
      json.diagnostics.filter((diagnostic: { supported: boolean }) => !diagnostic.supported)
    ).toHaveLength(unsupported);
    expect(html).toContain('Unsupported Diagnostics');
  });

  it('returns nonzero for malformed JSONL without writing reports', async () => {
    outputDirectory = await mkdtemp(join(tmpdir(), 'malformed-diagnostics-'));
    const input = join(outputDirectory, 'bad.jsonl');
    const jsonOut = join(outputDirectory, 'bad.json');
    const htmlOut = join(outputDirectory, 'bad.html');
    await writeFile(input, '{"reason":"compiler-artifact"}\n{bad}', 'utf8');

    const exitCode = await main(['--input', input, '--json-out', jsonOut, '--html-out', htmlOut]);

    expect(exitCode).toBe(1);
  });
});
