import type {
  DiagnosticSpan,
  DiagnosticSpanRole,
  Evidence
} from '../diagnostics/diagnostic-span.js';
import type { DiagnosticChildRecord, DiagnosticRecord, OwnershipEvent } from './ownership-event.js';
import { extractRustcSuggestions } from './rustc-suggestions.js';

export function mapE0382Diagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.code !== 'E0382') {
    return diagnostic;
  }

  const place = extractPlace(diagnostic.message);
  const spans = diagnostic.spans.map(classifyE0382Span);
  const children = (diagnostic.children ?? []).map(classifySuggestionChild);
  const classifiedDiagnostic = { ...diagnostic, spans, children };
  const spanEvents = spans.flatMap((span) => createSpanEvent(classifiedDiagnostic.id, span, place));
  const suggestionEvents = extractRustcSuggestions(classifiedDiagnostic).map(
    (suggestion, index) => ({
      id: `${classifiedDiagnostic.id}-event-${spanEvents.length + index + 1}`,
      diagnosticId: classifiedDiagnostic.id,
      kind: 'possible_fix' as const,
      role: 'possible_fix' as const,
      place,
      spanId: suggestion.span.id,
      message: suggestion.message,
      evidence: withDiagnosticCodeEvidence(suggestion.evidence),
      confidence: suggestion.span.confidence
    })
  );

  return {
    ...classifiedDiagnostic,
    supported: true,
    events: [...spanEvents, ...suggestionEvents]
  };
}

function classifyE0382Span(span: DiagnosticSpan): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (label.includes('value moved here')) {
    return { ...span, role: 'cause', confidence: 'high' };
  }

  if (label.includes('borrowed here after move') || label.includes('used here after move')) {
    return { ...span, role: 'conflict', confidence: 'high' };
  }

  if (label.includes('move occurs because')) {
    return { ...span, role: 'context', confidence: 'high' };
  }

  return span;
}

function classifySuggestionChild(child: DiagnosticChildRecord): DiagnosticChildRecord {
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

function createSpanEvent(
  diagnosticId: string,
  span: DiagnosticSpan,
  place: string | undefined
): OwnershipEvent[] {
  const eventKind = eventKindForRole(span.role, span.label);
  if (eventKind === null) {
    return [];
  }

  return [
    {
      id: `${diagnosticId}-event-${eventOrder(span.role)}`,
      diagnosticId,
      kind: eventKind,
      role: span.role,
      place,
      spanId: span.id,
      message: span.label ?? eventKind,
      evidence: withDiagnosticCodeEvidence(span.evidence),
      confidence: span.confidence
    }
  ];
}

function eventKindForRole(role: DiagnosticSpanRole, label: string | null | undefined) {
  if (role === 'cause') {
    return 'move' as const;
  }

  if (role === 'conflict') {
    return 'use' as const;
  }

  if (role === 'context') {
    return 'context' as const;
  }

  if (role === 'possible_fix') {
    return 'possible_fix' as const;
  }

  return label !== null && label !== undefined ? ('unknown' as const) : null;
}

function eventOrder(role: DiagnosticSpanRole): number {
  switch (role) {
    case 'cause':
      return 1;
    case 'conflict':
      return 2;
    case 'context':
      return 3;
    case 'possible_fix':
      return 4;
    default:
      return 99;
  }
}

function withDiagnosticCodeEvidence(evidence: Evidence[]): Evidence[] {
  return [{ source: 'diagnostic_code', field: 'code', value: 'E0382' }, ...evidence];
}

function extractPlace(message: string): string | undefined {
  return /`([^`]+)`/.exec(message)?.[1];
}
