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
    ...wordingForAudience(audience, {
      beginner: {
        whatHappened: `${place} was moved, and the code later tried to use it again.`,
        whyItMatters:
          'After a move, the original binding no longer owns the value, so Rust prevents later use through that binding.',
        nextStep:
          'Start at the move location, then decide whether the later code should borrow the value, clone it, or change ownership flow.'
      },
      intermediate: {
        whatHappened: `${place} crosses an ownership boundary before a later use still expects the original binding to be valid.`,
        whyItMatters:
          'The current API or control flow transfers ownership earlier than the later use permits.',
        nextStep:
          'Check the ownership boundary: prefer borrowing at the callee boundary, move the ownership transfer later, or make cloning an explicit cost.'
      },
      agent: {
        whatHappened: `move/use conflict for ${place}.`,
        whyItMatters: 'Original binding is invalid after move.',
        nextStep:
          'Repair hint: inspect cause span, then choose borrow, delayed move, or explicit clone.'
      }
    }),
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
    ...wordingForAudience(audience, {
      beginner: {
        whatHappened: `${place} already has an active mutable borrow, and the code requested another mutable borrow before the first one ended.`,
        whyItMatters:
          'Rust allows only one active mutable borrow at a time so writes cannot race or invalidate each other.',
        nextStep:
          'Find where the first mutable borrow is last used, then shorten that scope or reorder the second mutable borrow after it.'
      },
      intermediate: {
        whatHappened: `${place} has overlapping mutable-borrow scopes.`,
        whyItMatters:
          'The borrow scopes overlap across an operation boundary, so the second mutable access cannot be proven exclusive.',
        nextStep:
          'Shorten the first borrow scope, split the operation into phases, or move the second mutable access behind the first borrow boundary.'
      },
      agent: {
        whatHappened: `overlapping mutable borrows for ${place}.`,
        whyItMatters: 'Exclusive mutable access is violated.',
        nextStep:
          'Repair hint: end first borrow before second mutable borrow; split scope or reorder statements.'
      }
    }),
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
    ...wordingForAudience(audience, {
      beginner: {
        whatHappened: `${place} is still immutably borrowed when the code tries to borrow it mutably.`,
        whyItMatters:
          'Rust keeps shared reads and mutable writes from overlapping so readers cannot observe a value while it is being changed.',
        nextStep:
          'Locate the last use of the immutable borrow, then move the mutable borrow after that point or narrow the immutable borrow scope.'
      },
      intermediate: {
        whatHappened: `${place} has a shared borrow whose lifetime overlaps a mutable borrow request.`,
        whyItMatters:
          'The immutable reference remains live across the mutation boundary, so the mutable borrow cannot be exclusive.',
        nextStep:
          'Narrow the shared borrow lifetime, copy out the needed data, or move the mutation after the last shared use.'
      },
      agent: {
        whatHappened: `shared/mutable borrow overlap for ${place}.`,
        whyItMatters: 'Mutable borrow conflicts with live shared borrow.',
        nextStep:
          'Repair hint: end shared borrow before mutable borrow; narrow lifetime or reorder mutation.'
      }
    }),
    conceptTerms: ['immutable borrow', 'mutable borrow', 'borrow scope'],
    evidence: collectSummaryEvidence(diagnostic, [sharedBorrow, mutableBorrow]),
    confidence: summaryConfidence([sharedBorrow, mutableBorrow])
  };
}

interface AudienceWording {
  whatHappened: string;
  whyItMatters: string;
  nextStep: string;
}

function wordingForAudience(
  audience: AudienceMode,
  wording: Record<AudienceMode, AudienceWording>
): AudienceWording {
  return wording[audience];
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
