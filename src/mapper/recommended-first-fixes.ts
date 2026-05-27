import type { Evidence } from '../diagnostics/diagnostic-span.js';
import type { RecommendedFirstFix } from './diagnostic-navigation.js';
import type { DiagnosticRecord } from './ownership-event.js';

const FIRST_FIX_PRIORITY: Record<string, number> = {
  E0425: 10,
  E0308: 20,
  E0004: 30,
  E0382: 40,
  E0499: 50,
  E0502: 60
};

export function createRecommendedFirstFixes(
  diagnostics: readonly DiagnosticRecord[]
): RecommendedFirstFix[] {
  if (diagnostics.length < 2) {
    return [];
  }

  return diagnostics
    .map((diagnostic, index) => ({ diagnostic, index, rank: rankDiagnostic(diagnostic) }))
    .filter((entry) => entry.diagnostic.supported && entry.rank !== Number.POSITIVE_INFINITY)
    .sort((left, right) => left.rank - right.rank || left.index - right.index)
    .map((entry, priority) => createRecommendedFirstFix(entry.diagnostic, priority + 1));
}

function rankDiagnostic(diagnostic: DiagnosticRecord): number {
  return diagnostic.code === null || diagnostic.code === undefined
    ? Number.POSITIVE_INFINITY
    : (FIRST_FIX_PRIORITY[diagnostic.code] ?? Number.POSITIVE_INFINITY);
}

function createRecommendedFirstFix(
  diagnostic: DiagnosticRecord,
  priority: number
): RecommendedFirstFix {
  const strategy = diagnostic.fixStrategies?.[0];

  return {
    diagnosticId: diagnostic.id,
    code: diagnostic.code,
    priority,
    reason: firstFixReason(diagnostic),
    nextStep: strategy?.title ?? fallbackNextStep(diagnostic),
    evidence: firstFixEvidence(diagnostic),
    confidence: strategy?.confidence ?? diagnostic.events?.[0]?.confidence ?? 'medium'
  };
}

function firstFixReason(diagnostic: DiagnosticRecord): string {
  switch (diagnostic.code) {
    case 'E0425':
      return 'Resolve missing names first because later diagnostics may depend on the referenced item.';
    case 'E0308':
      return 'Align types before changing ownership so the code has the expected shape.';
    case 'E0004':
      return 'Complete match coverage before refining the surrounding control flow.';
    case 'E0382':
      return 'Fix the move/use conflict before changing later borrow behavior.';
    case 'E0499':
      return 'Remove overlapping mutable borrows so each mutation has exclusive access.';
    case 'E0502':
      return 'Separate shared and mutable access before applying local rewrites.';
    default:
      return 'Start with this supported diagnostic before unsupported follow-up errors.';
  }
}

function fallbackNextStep(diagnostic: DiagnosticRecord): string {
  switch (diagnostic.code) {
    case 'E0425':
      return 'Define, import, or rename the missing item.';
    case 'E0308':
      return 'Compare the expected type with the expression type at the primary span.';
    case 'E0004':
      return 'Add an explicit match arm or a wildcard arm for the missing case.';
    case 'E0382':
      return 'Inspect the move span and the later use span together.';
    case 'E0499':
    case 'E0502':
      return 'Find the last use of the earlier borrow and shorten that scope.';
    default:
      return 'Open the diagnostic span and apply the first available strategy.';
  }
}

function firstFixEvidence(diagnostic: DiagnosticRecord): Evidence[] {
  return (
    diagnostic.fixStrategies?.[0]?.evidence ??
    diagnostic.events?.[0]?.evidence ?? [
      { source: 'diagnostic_code', field: 'code', value: diagnostic.code ?? null }
    ]
  );
}
