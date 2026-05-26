import type {
  DiagnosticSpan,
  DiagnosticSpanRole,
  Evidence
} from '../diagnostics/diagnostic-span.js';
import type { DiagnosticRecord, OwnershipEvent, OwnershipEventKind } from './ownership-event.js';

export function mapE0499Diagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.code !== 'E0499') {
    return diagnostic;
  }

  const place = extractPlace(diagnostic.message);
  const spans = diagnostic.spans.map(classifyE0499Span);

  return {
    ...diagnostic,
    supported: true,
    spans,
    events: spans.flatMap((span, index) => createEvent(diagnostic.id, span, index, place))
  };
}

function classifyE0499Span(span: DiagnosticSpan): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (label.includes('first mutable borrow occurs here')) {
    return { ...span, role: 'cause', confidence: 'high' };
  }

  if (label.includes('second mutable borrow occurs here')) {
    return { ...span, role: 'conflict', confidence: 'high' };
  }

  if (label.includes('first borrow later used here')) {
    return { ...span, role: 'context', confidence: 'high' };
  }

  return span;
}

function createEvent(
  diagnosticId: string,
  span: DiagnosticSpan,
  index: number,
  place: string | undefined
): OwnershipEvent[] {
  const kind = kindForRole(span.role);
  if (kind === null) {
    return [];
  }

  return [
    {
      id: `${diagnosticId}-event-${index + 1}`,
      diagnosticId,
      kind,
      role: span.role,
      place,
      spanId: span.id,
      message: span.label ?? kind,
      evidence: withDiagnosticCodeEvidence(span.evidence),
      confidence: span.confidence
    }
  ];
}

function kindForRole(role: DiagnosticSpanRole): OwnershipEventKind | null {
  if (role === 'cause') {
    return 'borrow_mut';
  }

  if (role === 'conflict') {
    return 'borrow_mut_request';
  }

  if (role === 'context') {
    return 'context';
  }

  return null;
}

function withDiagnosticCodeEvidence(evidence: Evidence[]): Evidence[] {
  return [{ source: 'diagnostic_code', field: 'code', value: 'E0499' }, ...evidence];
}

function extractPlace(message: string): string | undefined {
  return /`([^`]+)`/.exec(message)?.[1];
}
