import type { Evidence } from '../diagnostics/diagnostic-span.js';
import type { AudienceMode, LearnerSummary } from './diagnostic-navigation.js';
import type { DiagnosticRecord, OwnershipEvent } from './ownership-event.js';

export function createLearnerSummary(
  diagnostic: DiagnosticRecord,
  audience: AudienceMode = 'beginner'
): LearnerSummary | undefined {
  if (!diagnostic.supported) {
    return undefined;
  }

  switch (diagnostic.code) {
    case 'E0382':
      return createE0382Summary(diagnostic, audience);
    case 'E0499':
      return createE0499Summary(diagnostic, audience);
    case 'E0502':
      return createE0502Summary(diagnostic, audience);
    default:
      return undefined;
  }
}

export function attachLearnerSummary(
  diagnostic: DiagnosticRecord,
  audience: AudienceMode = 'beginner'
): DiagnosticRecord {
  const learnerSummary = createLearnerSummary(diagnostic, audience);

  return learnerSummary === undefined ? diagnostic : { ...diagnostic, learnerSummary };
}

function createE0382Summary(diagnostic: DiagnosticRecord, audience: AudienceMode): LearnerSummary {
  const moveEvent = findEvent(diagnostic, 'move', 'cause');
  const useEvent = findEvent(diagnostic, 'use', 'conflict');
  const place = moveEvent?.place ?? useEvent?.place ?? 'the value';

  return {
    audience,
    whatHappened: `
      ${place} was moved, and the code later tried to use it again.
    `.trim(),
    whyItMatters:
      'After a move, the original binding no longer owns the value, so Rust prevents later use through that binding.',
    nextStep:
      'Start at the move location, then decide whether the later code should borrow the value, clone it, or change ownership flow.',
    conceptTerms: ['move', 'ownership', 'use after move'],
    evidence: collectSummaryEvidence(diagnostic, [moveEvent, useEvent]),
    confidence: summaryConfidence([moveEvent, useEvent])
  };
}

function createE0499Summary(diagnostic: DiagnosticRecord, audience: AudienceMode): LearnerSummary {
  const firstBorrow = findEvent(diagnostic, 'borrow_mut', 'cause');
  const secondBorrow = findEvent(diagnostic, 'borrow_mut_request', 'conflict');
  const place = firstBorrow?.place ?? secondBorrow?.place ?? 'the value';

  return {
    audience,
    whatHappened: `
      ${place} already has an active mutable borrow, and the code requested another mutable borrow before the first one ended.
    `.trim(),
    whyItMatters:
      'Rust allows only one active mutable borrow at a time so writes cannot race or invalidate each other.',
    nextStep:
      'Find where the first mutable borrow is last used, then shorten that scope or reorder the second mutable borrow after it.',
    conceptTerms: ['mutable borrow', 'borrow scope', 'conflict'],
    evidence: collectSummaryEvidence(diagnostic, [firstBorrow, secondBorrow]),
    confidence: summaryConfidence([firstBorrow, secondBorrow])
  };
}

function createE0502Summary(diagnostic: DiagnosticRecord, audience: AudienceMode): LearnerSummary {
  const sharedBorrow = findEvent(diagnostic, 'borrow_shared', 'cause');
  const mutableBorrow = findEvent(diagnostic, 'borrow_mut', 'conflict');
  const place = sharedBorrow?.place ?? mutableBorrow?.place ?? 'the value';

  return {
    audience,
    whatHappened: `
      ${place} is still immutably borrowed when the code tries to borrow it mutably.
    `.trim(),
    whyItMatters:
      'Rust keeps shared reads and mutable writes from overlapping so readers cannot observe a value while it is being changed.',
    nextStep:
      'Locate the last use of the immutable borrow, then move the mutable borrow after that point or narrow the immutable borrow scope.',
    conceptTerms: ['immutable borrow', 'mutable borrow', 'borrow scope'],
    evidence: collectSummaryEvidence(diagnostic, [sharedBorrow, mutableBorrow]),
    confidence: summaryConfidence([sharedBorrow, mutableBorrow])
  };
}

function findEvent(
  diagnostic: DiagnosticRecord,
  kind: OwnershipEvent['kind'],
  role: OwnershipEvent['role']
): OwnershipEvent | undefined {
  return diagnostic.events?.find((event) => event.kind === kind && event.role === role);
}

function collectSummaryEvidence(
  diagnostic: DiagnosticRecord,
  events: readonly (OwnershipEvent | undefined)[]
): Evidence[] {
  const evidence = events
    .flatMap((event) => event?.evidence ?? [])
    .filter((evidenceItem) => isSummarySafeEvidenceSource(evidenceItem.source));

  return evidence.length > 0
    ? evidence
    : [{ source: 'diagnostic_code', field: 'code', value: diagnostic.code ?? null }];
}

function isSummarySafeEvidenceSource(source: Evidence['source']): boolean {
  return source !== 'rustc_span_text' && source !== 'rustc_suggestion';
}

function summaryConfidence(events: readonly (OwnershipEvent | undefined)[]) {
  const foundEvents = events.filter((event) => event !== undefined);
  if (
    foundEvents.length === events.length &&
    foundEvents.every((event) => event.confidence === 'high')
  ) {
    return 'high' as const;
  }

  return foundEvents.length > 0 ? ('medium' as const) : ('low' as const);
}
