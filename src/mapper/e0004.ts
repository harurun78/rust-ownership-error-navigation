import type { DiagnosticSpan } from '../diagnostics/diagnostic-span.js';
import type { DiagnosticChildRecord, DiagnosticRecord } from './ownership-event.js';
import { createNonOwnershipFixStrategy } from './non-ownership-helpers.js';

export function mapE0004Diagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  if (diagnostic.code !== 'E0004') {
    return diagnostic;
  }

  const spans = diagnostic.spans.map(classifyE0004Span);
  const children = (diagnostic.children ?? []).map(classifyE0004Child);
  const classifiedDiagnostic = { ...diagnostic, spans, children };

  return {
    ...classifiedDiagnostic,
    supported: true,
    fixStrategies: [
      createNonOwnershipFixStrategy({
        diagnostic: classifiedDiagnostic,
        kind: 'add_match_arm',
        title: 'Cover every possible match case',
        rationale: 'Rust cannot prove this match handles every value the input type can produce.',
        tradeOffs: [
          'Adding an explicit arm documents the missing case clearly.',
          'A wildcard arm is shorter but can hide future enum variants.'
        ],
        fallbackSpan: spans.find((span) => span.role === 'conflict')
      })
    ]
  };
}

function classifyE0004Span(span: DiagnosticSpan): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (span.isPrimary && label.includes('not covered')) {
    return { ...span, role: 'conflict', confidence: 'high' };
  }

  return span;
}

function classifyE0004Child(child: DiagnosticChildRecord): DiagnosticChildRecord {
  const message = child.message.toLowerCase();

  return {
    ...child,
    spans: child.spans.map((span) => classifyE0004ChildSpan(span, message)),
    children: child.children?.map(classifyE0004Child)
  };
}

function classifyE0004ChildSpan(span: DiagnosticSpan, childMessage: string): DiagnosticSpan {
  const label = span.label?.toLowerCase() ?? '';

  if (span.suggestedReplacement !== null && span.suggestedReplacement !== undefined) {
    return { ...span, role: 'possible_fix', confidence: 'high' };
  }

  if (label.includes('not covered')) {
    return { ...span, role: 'cause', confidence: 'high' };
  }

  if (childMessage.includes('defined here') || label.length === 0) {
    return { ...span, role: 'context', confidence: 'medium' };
  }

  return span;
}
