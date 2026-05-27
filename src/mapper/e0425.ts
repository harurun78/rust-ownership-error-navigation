import type { DiagnosticSpan } from '../diagnostics/diagnostic-span.js';
import type { DiagnosticRecord } from './ownership-event.js';
import { classifySuggestionChild, createNonOwnershipFixStrategy } from './non-ownership-helpers.js';

export function mapE0425Diagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.code !== 'E0425') {
    return diagnostic;
  }

  const spans = diagnostic.spans.map(classifyE0425Span);
  const children = (diagnostic.children ?? []).map(classifySuggestionChild);
  const classifiedDiagnostic = { ...diagnostic, spans, children };

  return {
    ...classifiedDiagnostic,
    supported: true,
    fixStrategies: [
      createNonOwnershipFixStrategy({
        diagnostic: classifiedDiagnostic,
        kind: 'resolve_name',
        title: 'Resolve the missing name in this scope',
        rationale: 'Rust could not find the referenced item where this code is compiled.',
        tradeOffs: [
          'Renaming the reference is direct when this is a typo.',
          'Adding an import or declaration is better when the item exists elsewhere.'
        ],
        fallbackSpan: spans.find((span) => span.role === 'conflict')
      })
    ]
  };
}

function classifyE0425Span(span: DiagnosticSpan): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (span.isPrimary && (label.includes('not found') || label.includes('scope'))) {
    return { ...span, role: 'conflict', confidence: 'high' };
  }

  return span;
}
