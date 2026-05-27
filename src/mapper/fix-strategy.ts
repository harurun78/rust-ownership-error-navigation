import type { Evidence } from '../diagnostics/diagnostic-span.js';
import type { FixStrategy, FixStrategyKind } from './diagnostic-navigation.js';
import type { DiagnosticRecord, OwnershipEvent } from './ownership-event.js';

export function attachFixStrategies(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.fixStrategies !== undefined && diagnostic.fixStrategies.length > 0) {
    return diagnostic;
  }

  const fixStrategies = createOwnershipFixStrategies(diagnostic);

  return fixStrategies.length === 0 ? diagnostic : { ...diagnostic, fixStrategies };
}

export function createOwnershipFixStrategies(diagnostic: DiagnosticRecord): FixStrategy[] {
  switch (diagnostic.code) {
    case 'E0382':
      return createE0382Strategies(diagnostic);
    case 'E0499':
      return createE0499Strategies(diagnostic);
    case 'E0502':
      return createE0502Strategies(diagnostic);
    default:
      return [];
  }
}

function createE0382Strategies(diagnostic: DiagnosticRecord): FixStrategy[] {
  const moveEvent = findEvent(diagnostic, 'move', 'cause');
  const useEvent = findEvent(diagnostic, 'use', 'conflict');
  const cloneEvent = findEvent(diagnostic, 'possible_fix', 'possible_fix');

  return [
    createStrategy({
      diagnostic,
      index: 1,
      kind: 'borrow',
      title: 'Borrow the value instead of moving it',
      rationale:
        'If later code still needs the original binding, pass a reference at the ownership boundary.',
      tradeOffs: [
        'Keeps one owner and avoids an allocation.',
        'Requires the called API to accept a reference or a borrowed variant.'
      ],
      event: moveEvent ?? useEvent
    }),
    createStrategy({
      diagnostic,
      index: 2,
      kind: 'clone_or_copy',
      title: 'Use clone() only when two owned values are intentional',
      rationale:
        'Cloning can satisfy ownership when both locations truly need independent owned values.',
      tradeOffs: [
        'Makes the extra allocation or copy explicit.',
        'Can hide a design issue if borrowing or moving later would express the intent better.'
      ],
      event: cloneEvent ?? useEvent
    }),
    createStrategy({
      diagnostic,
      index: 3,
      kind: 'move_timing',
      title: 'Move ownership after the last use',
      rationale:
        'Reordering can keep the original binding valid until the final read has happened.',
      tradeOffs: [
        'Often keeps APIs unchanged.',
        'May require splitting a larger expression into smaller statements.'
      ],
      event: useEvent ?? moveEvent
    })
  ];
}

function createE0499Strategies(diagnostic: DiagnosticRecord): FixStrategy[] {
  const firstBorrow = findEvent(diagnostic, 'borrow_mut', 'cause');
  const secondBorrow = findEvent(diagnostic, 'borrow_mut_request', 'conflict');
  const laterUse = findEvent(diagnostic, 'context', 'context');

  return [
    createStrategy({
      diagnostic,
      index: 1,
      kind: 'split_scope',
      title: 'Shorten the first mutable borrow scope',
      rationale:
        'End the first mutable borrow before requesting another mutable borrow of the same value.',
      tradeOffs: [
        'Keeps exclusive access clear to the compiler.',
        'May require introducing a smaller block or moving the last use earlier.'
      ],
      event: laterUse ?? firstBorrow
    }),
    createStrategy({
      diagnostic,
      index: 2,
      kind: 'move_timing',
      title: 'Reorder the second mutable operation after the first borrow ends',
      rationale:
        'The second mutable borrow is valid once the first mutable reference is no longer used.',
      tradeOffs: [
        'Preserves mutation semantics without cloning.',
        'Can change behavior if the two operations were intentionally interleaved.'
      ],
      event: secondBorrow ?? laterUse
    })
  ];
}

function createE0502Strategies(diagnostic: DiagnosticRecord): FixStrategy[] {
  const sharedBorrow = findEvent(diagnostic, 'borrow_shared', 'cause');
  const mutableBorrow = findEvent(diagnostic, 'borrow_mut', 'conflict');
  const laterUse = findEvent(diagnostic, 'context', 'context');

  return [
    createStrategy({
      diagnostic,
      index: 1,
      kind: 'split_scope',
      title: 'End the shared borrow before the mutable borrow',
      rationale:
        'The mutable operation must happen after the immutable reference is no longer needed.',
      tradeOffs: [
        'Usually avoids extra allocation and keeps aliasing rules explicit.',
        'May require copying out a small value or limiting a reference to a smaller block.'
      ],
      event: laterUse ?? sharedBorrow
    }),
    createStrategy({
      diagnostic,
      index: 2,
      kind: 'move_timing',
      title: 'Move the mutation after the last shared read',
      rationale: 'Operation ordering can remove the overlap between shared and mutable access.',
      tradeOffs: [
        'Keeps the data model unchanged.',
        'Requires checking that the reordered mutation still matches the intended behavior.'
      ],
      event: mutableBorrow ?? laterUse
    })
  ];
}

function createStrategy(options: {
  diagnostic: DiagnosticRecord;
  index: number;
  kind: FixStrategyKind;
  title: string;
  rationale: string;
  tradeOffs: string[];
  event?: OwnershipEvent;
}): FixStrategy {
  return {
    id: `${options.diagnostic.id}-fix-${options.index}`,
    diagnosticId: options.diagnostic.id,
    kind: options.kind,
    title: options.title,
    rationale: options.rationale,
    tradeOffs: options.tradeOffs,
    ...(options.event === undefined ? {} : { spanId: options.event.spanId }),
    evidence: strategyEvidence(options.diagnostic, options.event),
    confidence: options.event?.confidence ?? 'medium'
  };
}

function findEvent(
  diagnostic: DiagnosticRecord,
  kind: OwnershipEvent['kind'],
  role: OwnershipEvent['role']
): OwnershipEvent | undefined {
  return diagnostic.events?.find((event) => event.kind === kind && event.role === role);
}

function strategyEvidence(
  diagnostic: DiagnosticRecord,
  event: OwnershipEvent | undefined
): Evidence[] {
  return (
    event?.evidence ?? [
      { source: 'diagnostic_code', field: 'code', value: diagnostic.code ?? null }
    ]
  );
}
