import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostic } from '../../src/mapper/index.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('ownership fix strategies', () => {
  it('presents clone() as one E0382 strategy with trade-offs', async () => {
    const record = await mapFixtureDiagnostic('E0382');
    const cloneStrategy = record.fixStrategies?.find(
      (strategy) => strategy.kind === 'clone_or_copy'
    );

    expect(record.fixStrategies?.map((strategy) => strategy.kind)).toEqual([
      'borrow',
      'clone_or_copy',
      'move_timing'
    ]);
    expect(cloneStrategy).toMatchObject({
      title: 'Use clone() only when two owned values are intentional',
      confidence: 'high'
    });
    expect(cloneStrategy?.tradeOffs).toEqual([
      'Makes the extra allocation or copy explicit.',
      'Can hide a design issue if borrowing or moving later would express the intent better.'
    ]);
  });

  it('adds scope-shortening and ordering guidance for E0499', async () => {
    const record = await mapFixtureDiagnostic('E0499');

    expect(record.fixStrategies?.map((strategy) => [strategy.kind, strategy.title])).toEqual([
      ['split_scope', 'Shorten the first mutable borrow scope'],
      ['move_timing', 'Reorder the second mutable operation after the first borrow ends']
    ]);
    expect(record.fixStrategies?.every((strategy) => strategy.evidence.length > 0)).toBe(true);
  });

  it('adds scope-shortening and ordering guidance for E0502', async () => {
    const record = await mapFixtureDiagnostic('E0502');

    expect(record.fixStrategies?.map((strategy) => [strategy.kind, strategy.title])).toEqual([
      ['split_scope', 'End the shared borrow before the mutable borrow'],
      ['move_timing', 'Move the mutation after the last shared read']
    ]);
    expect(record.fixStrategies?.every((strategy) => strategy.evidence.length > 0)).toBe(true);
  });
});

async function mapFixtureDiagnostic(code: string) {
  const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
  const message = messages.find((item) => item.message?.code?.code === code);

  return mapDiagnostic(normalizeRustcDiagnostic(message!.message!, { diagnosticId: code }));
}
