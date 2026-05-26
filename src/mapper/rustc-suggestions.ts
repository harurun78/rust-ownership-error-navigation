import type { DiagnosticSpan, Evidence } from '../diagnostics/diagnostic-span.js';
import type { DiagnosticChildRecord, DiagnosticRecord } from './ownership-event.js';

export interface RustcSuggestion {
  message: string;
  span: DiagnosticSpan;
  evidence: Evidence[];
}

export function extractRustcSuggestions(diagnostic: DiagnosticRecord): RustcSuggestion[] {
  return (diagnostic.children ?? []).flatMap((child) => extractChildSuggestions(child));
}

function extractChildSuggestions(child: DiagnosticChildRecord): RustcSuggestion[] {
  const childEvidence: Evidence = {
    source: 'rustc_child_diagnostic',
    field: 'message',
    value: child.message
  };
  const suggestions = child.spans
    .filter((span) => span.suggestedReplacement !== null && span.suggestedReplacement !== undefined)
    .map((span) => ({
      message: child.message,
      span,
      evidence: [childEvidence, ...span.evidence]
    }));

  return [
    ...suggestions,
    ...(child.children ?? []).flatMap((nested) => extractChildSuggestions(nested))
  ];
}
