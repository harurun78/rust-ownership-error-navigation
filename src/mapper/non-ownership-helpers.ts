import type { DiagnosticSpan, Evidence } from '../diagnostics/diagnostic-span.js';
import type { FixStrategy, FixStrategyKind } from './diagnostic-navigation.js';
import type { DiagnosticChildRecord } from './ownership-event.js';
import { extractRustcSuggestions } from './rustc-suggestions.js';
import type { DiagnosticRecord } from './ownership-event.js';

export function classifySuggestionChild(child: DiagnosticChildRecord): DiagnosticChildRecord {
  return {
    ...child,
    spans: child.spans.map((span) =>
      span.suggestedReplacement !== null && span.suggestedReplacement !== undefined
        ? { ...span, role: 'possible_fix', confidence: 'high' }
        : span
    ),
    children: child.children?.map(classifySuggestionChild)
  };
}

export function createNonOwnershipFixStrategy(options: {
  diagnostic: DiagnosticRecord;
  kind: FixStrategyKind;
  title: string;
  rationale: string;
  tradeOffs: string[];
  fallbackSpan?: DiagnosticSpan;
}): FixStrategy {
  const suggestions = extractRustcSuggestions(options.diagnostic);
  const suggestion = suggestions[0];
  const span = suggestion?.span ?? options.fallbackSpan ?? options.diagnostic.spans[0];

  return {
    id: `${options.diagnostic.id}-fix-1`,
    diagnosticId: options.diagnostic.id,
    kind: options.kind,
    title: options.title,
    rationale: options.rationale,
    tradeOffs: options.tradeOffs,
    ...(span === undefined ? {} : { spanId: span.id }),
    evidence: withDiagnosticCodeEvidence(
      options.diagnostic.code,
      suggestion?.evidence ?? span?.evidence ?? []
    ),
    confidence: suggestion?.span.confidence ?? span?.confidence ?? 'medium'
  };
}

export function withDiagnosticCodeEvidence(
  code: string | null | undefined,
  evidence: Evidence[]
): Evidence[] {
  return [{ source: 'diagnostic_code', field: 'code', value: code ?? null }, ...evidence];
}
