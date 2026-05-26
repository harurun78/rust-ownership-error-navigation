import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('rustc diagnostic normalizer', () => {
  it('preserves diagnostic code, level, message, rendered output, and spans', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostic = message?.message;

    expect(diagnostic).toBeDefined();

    const record = normalizeRustcDiagnostic(diagnostic!, { diagnosticIndex: 0 });

    expect(record).toMatchObject({
      id: 'diagnostic-1',
      code: 'E0382',
      supported: false,
      level: 'error',
      message: 'borrow of moved value: `s`'
    });
    expect(record.rendered).toContain('error[E0382]');
    expect(record.spans).toHaveLength(3);
  });

  it('keeps rustc line and column locations 1-based', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const record = normalizeRustcDiagnostic(message!.message!, { diagnosticIndex: 0 });

    expect(record.spans[0]).toMatchObject({
      file: 'src/main.rs',
      lineStart: 7,
      lineEnd: 7,
      columnStart: 21,
      columnEnd: 22,
      byteStart: 118,
      byteEnd: 119
    });
  });

  it('preserves child diagnostics and suggestion span metadata', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const record = normalizeRustcDiagnostic(message!.message!, { diagnosticIndex: 0 });

    expect(record.children).toHaveLength(2);
    expect(record.children?.[1]).toMatchObject({
      level: 'help',
      message: 'consider cloning the value if the performance cost is acceptable'
    });
    expect(record.children?.[1]?.spans[0]).toMatchObject({
      id: 'diagnostic-1-child-2-span-1',
      suggestedReplacement: '.clone()',
      suggestionApplicability: 'MachineApplicable',
      lineStart: 7,
      columnStart: 22
    });
  });

  it('marks spans with macro expansion while retaining snippets and null labels', async () => {
    const [message] = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const record = normalizeRustcDiagnostic(message!.message!, { diagnosticIndex: 0 });

    expect(record.spans[1]).toMatchObject({
      hasExpansion: true,
      label: 'value borrowed here after move',
      snippet: '    println!("{}", s);'
    });
    expect(record.children?.[1]?.spans[0]?.label).toBeNull();
  });
});
