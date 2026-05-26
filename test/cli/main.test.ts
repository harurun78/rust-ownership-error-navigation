import { describe, expect, it } from 'vitest';

import { parseCliOptions } from '../../src/cli/main.js';

describe('CLI option parsing', () => {
  it('parses required report paths', () => {
    expect(
      parseCliOptions([
        '--input',
        'diagnostics.jsonl',
        '--json-out',
        'report.json',
        '--html-out',
        'report.html'
      ])
    ).toEqual({
      input: 'diagnostics.jsonl',
      jsonOut: 'report.json',
      htmlOut: 'report.html'
    });
  });

  it('rejects options without values', () => {
    expect(() => parseCliOptions(['--input'])).toThrow('Missing value for --input.');
  });

  it('rejects the next option token as a value', () => {
    expect(() =>
      parseCliOptions(['--input', '--json-out', 'report.json', '--html-out', 'report.html'])
    ).toThrow('Missing value for --input.');
  });
});
