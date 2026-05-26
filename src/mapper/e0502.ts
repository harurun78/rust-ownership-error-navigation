import type {
  DiagnosticSpan,
  DiagnosticSpanRole,
  Evidence
} from '../diagnostics/diagnostic-span.js';
import type { DiagnosticRecord, OwnershipEvent, OwnershipEventKind } from './ownership-event.js';

export function mapE0502Diagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.code !== 'E0502') {
    return diagnostic;
  }

  const place = extractPlace(diagnostic.message);
  const spans = diagnostic.spans.map(classifyE0502Span);

  return {
    ...diagnostic,
    supported: true,
    spans,
    events: spans.flatMap((span, index) => createEvent(diagnostic.id, span, index, place))
  };
}

function classifyE0502Span(span: DiagnosticSpan): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (label.includes('immutable borrow occurs here')) {
    return { ...span, role: 'cause', confidence: 'high' };
  }

  if (label.includes('mutable borrow occurs here')) {
    return { ...span, role: 'conflict', confidence: 'high' };
  }

  if (label.includes('immutable borrow later used here')) {
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
    return 'borrow_shared';
  }

  if (role === 'conflict') {
    return 'borrow_mut';
  }

  if (role === 'context') {
    return 'context';
  }

  return null;
}

function withDiagnosticCodeEvidence(evidence: Evidence[]): Evidence[] {
  return [{ source: 'diagnostic_code', field: 'code', value: 'E0502' }, ...evidence];
}

function extractPlace(message: string): string | undefined {
  return /`([^`]+)`/.exec(message)?.[1];
}
