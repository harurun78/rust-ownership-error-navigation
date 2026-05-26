export type DiagnosticSpanRole = 'conflict' | 'cause' | 'context' | 'possible_fix' | 'unknown';

export type Confidence = 'high' | 'medium' | 'low';

export type EvidenceSource =
  | 'diagnostic_code'
  | 'rustc_primary_span'
  | 'rustc_span_label'
  | 'rustc_child_diagnostic'
  | 'rustc_suggestion'
  | 'rustc_span_text'
  | 'rustc_expansion'
  | 'heuristic';

export interface Evidence {
  source: EvidenceSource;
  field: string;
  value?: string | number | boolean | null;
}

export interface DiagnosticSpan {
  id: string;
  diagnosticId: string;
  role: DiagnosticSpanRole;
  file: string;
  lineStart: number;
  lineEnd: number;
  columnStart: number;
  columnEnd: number;
  byteStart?: number;
  byteEnd?: number;
  isPrimary: boolean;
  label?: string | null;
  snippet?: string;
  suggestedReplacement?: string | null;
  suggestionApplicability?: string | null;
  hasExpansion: boolean;
  evidence: Evidence[];
  confidence: Confidence;
}
