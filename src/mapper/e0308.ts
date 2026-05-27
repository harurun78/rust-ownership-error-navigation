import type { DiagnosticSpan } from '../diagnostics/diagnostic-span.js';
import type { DiagnosticRecord } from './ownership-event.js';
import { classifySuggestionChild, createNonOwnershipFixStrategy } from './non-ownership-helpers.js';

export function mapE0308Diagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.code !== 'E0308') {
    return diagnostic;
  }

  const spans = diagnostic.spans.map(classifyE0308Span);
  const children = (diagnostic.children ?? []).map(classifySuggestionChild);
  const classifiedDiagnostic = { ...diagnostic, spans, children };

  return {
    ...classifiedDiagnostic,
    supported: true,
    fixStrategies: [
      createNonOwnershipFixStrategy({
        diagnostic: classifiedDiagnostic,
        kind: 'align_types',
        title: 'Align the expression type with the expected type',
        rationale:
          'Rust expected one type at this location but inferred a different expression type.',
        tradeOffs: [
          'Changing the expression may be simpler when the expected type is correct.',
          'Changing the annotation or signature may be better when the expected type is too narrow.'
        ],
        fallbackSpan: spans.find((span) => span.role === 'conflict')
      })
    ]
  };
}

function classifyE0308Span(span: DiagnosticSpan): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (span.isPrimary && (label.includes('expected') || label.includes('found'))) {
    return { ...span, role: 'conflict', confidence: 'high' };
  }

  if (label.includes('expected due to this')) {
    return { ...span, role: 'cause', confidence: 'high' };
  }

  if (label.includes('expected')) {
    return { ...span, role: 'context', confidence: 'medium' };
  }

  return span;
}
