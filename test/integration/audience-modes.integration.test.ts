import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import { main } from '../../src/cli/main.js';
import { diagnosticFixturePath } from '../helpers/diagnostic-fixtures.js';

let outputDirectory: string | undefined;

describe('audience mode CLI integration', () => {
  afterEach(async () => {
    if (outputDirectory !== undefined) {
      await rm(outputDirectory, { recursive: true, force: true });
      outputDirectory = undefined;
    }
  });

  it('writes beginner, intermediate, and agent report variants', async () => {
    outputDirectory = await mkdtemp(join(tmpdir(), 'ownership-audience-'));
    const reports = await Promise.all(
      (['beginner', 'intermediate', 'agent'] as const).map(async (audience) => {
        const jsonOut = join(outputDirectory!, `${audience}.json`);
        const htmlOut = join(outputDirectory!, `${audience}.html`);
        const exitCode = await main([
          '--input',
          diagnosticFixturePath('ownership-baseline-2026-05-24.jsonl'),
          '--json-out',
          jsonOut,
          '--html-out',
          htmlOut,
          '--audience',
          audience
        ]);

        return {
          audience,
          exitCode,
          json: JSON.parse(await readFile(jsonOut, 'utf8')),
          html: await readFile(htmlOut, 'utf8')
        };
      })
    );

    expect(reports.map((report) => report.exitCode)).toEqual([0, 0, 0]);
    expect(reports.map((report) => report.json.input.audienceMode)).toEqual([
      'beginner',
      'intermediate',
      'agent'
    ]);
    expect(reports[0]!.html).toContain('Audience</th><td>beginner');
    expect(reports[1]!.html).toContain('ownership boundary');
    expect(reports[2]!.html).toContain('Repair hint:');
  });
});
