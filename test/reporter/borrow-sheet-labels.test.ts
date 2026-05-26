import { describe, expect, it } from 'vitest';

import { createBorrowSheetRows } from '../../src/reporter/borrow-sheet.js';
import type { DiagnosticRecord, OwnershipEventKind } from '../../src/mapper/ownership-event.js';

describe('Borrow Sheet labels', () => {
  it('preserves Rust event kind labels without conversion', () => {
    const kinds: OwnershipEventKind[] = [
      'move',
      'borrow_shared',
      'borrow_mut',
      'use',
      'conflict',
      'possible_fix',
      'context'
    ];
    const diagnostic: DiagnosticRecord = {
      id: 'diagnostic-labels',
      code: 'E0382',
      supported: true,
      message: 'labels',
      spans: [],
      events: kinds.map((kind, index) => ({
        id: `event-${index + 1}`,
        diagnosticId: 'diagnostic-labels',
        kind,
        role: kind === 'possible_fix' ? 'possible_fix' : kind === 'context' ? 'context' : 'unknown',
        spanId: `span-${index + 1}`,
        message: kind,
        evidence: [{ source: 'heuristic', field: 'test', value: kind }],
        confidence: 'high'
      }))
    };

    expect(createBorrowSheetRows([diagnostic]).map((row) => row.kind)).toEqual(kinds);
  });
});
