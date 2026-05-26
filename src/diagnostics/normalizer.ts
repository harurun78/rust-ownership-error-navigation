import type { DiagnosticSpan, Evidence } from './diagnostic-span.js';
import type { RustcDiagnostic, RustcSpan } from './rustc-diagnostic.js';
import type { DiagnosticChildRecord, DiagnosticRecord } from '../mapper/ownership-event.js';

export interface NormalizeDiagnosticOptions {
  diagnosticId?: string;
  diagnosticIndex?: number;
  supported?: boolean;
  unsupportedReason?: string;
}

export function normalizeRustcDiagnostic(
  diagnostic: RustcDiagnostic,
  options: NormalizeDiagnosticOptions = {}
): DiagnosticRecord {
  const diagnosticId = options.diagnosticId ?? `diagnostic-${(options.diagnosticIndex ?? 0) + 1}`;

  const record: DiagnosticRecord = {
    id: diagnosticId,
    code: diagnostic.code?.code ?? null,
    supported: options.supported ?? false,
    level: diagnostic.level,
    message: diagnostic.message,
    spans: normalizeRustcSpans(diagnostic.spans, diagnosticId),
    children: diagnostic.children.map((child, childIndex) =>
      normalizeRustcChildDiagnostic(child, `${diagnosticId}-child-${childIndex + 1}`)
    ),
    rendered: diagnostic.rendered ?? null
  };

  if (options.unsupportedReason !== undefined) {
    record.unsupportedReason = options.unsupportedReason;
  }

  return record;
}

export function normalizeRustcSpans(
  spans: readonly RustcSpan[],
  diagnosticId: string
): DiagnosticSpan[] {
  return spans.map((span, spanIndex) => normalizeRustcSpan(span, diagnosticId, spanIndex));
}

export function normalizeRustcSpan(
  span: RustcSpan,
  diagnosticId: string,
  spanIndex: number
): DiagnosticSpan {
  return {
    id: `${diagnosticId}-span-${spanIndex + 1}`,
    diagnosticId,
    role: 'unknown',
    file: span.file_name,
    lineStart: span.line_start,
    lineEnd: span.line_end,
    columnStart: span.column_start,
    columnEnd: span.column_end,
    byteStart: span.byte_start,
    byteEnd: span.byte_end,
    isPrimary: span.is_primary,
    label: span.label,
    snippet: span.text.map((text) => text.text).join('\n'),
    suggestedReplacement: span.suggested_replacement ?? null,
    suggestionApplicability: span.suggestion_applicability ?? null,
    hasExpansion: span.expansion !== null && span.expansion !== undefined,
    evidence: collectSpanEvidence(span),
    confidence: inferSpanConfidence(span)
  };
}

function normalizeRustcChildDiagnostic(
  diagnostic: RustcDiagnostic,
  diagnosticId: string
): DiagnosticChildRecord {
  return {
    code: diagnostic.code?.code ?? null,
    level: diagnostic.level,
    message: diagnostic.message,
    spans: normalizeRustcSpans(diagnostic.spans, diagnosticId),
    children: diagnostic.children.map((child, childIndex) =>
      normalizeRustcChildDiagnostic(child, `${diagnosticId}-child-${childIndex + 1}`)
    ),
    rendered: diagnostic.rendered ?? null
  };
}

function collectSpanEvidence(span: RustcSpan): Evidence[] {
  const evidence: Evidence[] = [];

  if (span.is_primary) {
    evidence.push({ source: 'rustc_primary_span', field: 'is_primary', value: true });
  }

  if (span.label !== null) {
    evidence.push({ source: 'rustc_span_label', field: 'label', value: span.label });
  }

  if (span.suggested_replacement !== null && span.suggested_replacement !== undefined) {
    evidence.push({
      source: 'rustc_suggestion',
      field: 'suggested_replacement',
      value: span.suggested_replacement
    });
  }

  if (span.text.length > 0) {
    evidence.push({ source: 'rustc_span_text', field: 'text', value: span.text[0]?.text ?? '' });
  }

  if (span.expansion !== null && span.expansion !== undefined) {
    evidence.push({ source: 'rustc_expansion', field: 'expansion', value: true });
  }

  if (evidence.length === 0) {
    evidence.push({ source: 'heuristic', field: 'span', value: 'unlabeled rustc span' });
  }

  return evidence;
}

function inferSpanConfidence(span: RustcSpan) {
  if (span.label !== null || span.suggested_replacement !== null) {
    return 'high';
  }

  if (span.is_primary) {
    return 'medium';
  }

  return 'low';
}
