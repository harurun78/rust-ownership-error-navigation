import type {
  Confidence,
  DiagnosticSpan,
  DiagnosticSpanRole,
  Evidence
} from '../diagnostics/diagnostic-span.js';
import type {
  AudienceMode,
  FixStrategy,
  LearnerSummary,
  RecommendedFirstFix
} from './diagnostic-navigation.js';

export type { AudienceMode, FixStrategy, LearnerSummary, RecommendedFirstFix };

export type OwnershipEventKind =
  | 'declare'
  | 'move'
  | 'move_out'
  | 'copy'
  | 'borrow_shared'
  | 'borrow_mut'
  | 'borrow_mut_request'
  | 'assign'
  | 'temporary'
  | 'escape'
  | 'closure_capture'
  | 'partial_move'
  | 'receiver_move'
  | 'implicit_into_iter'
  | 'async_send_boundary'
  | 'static_requirement'
  | 'use'
  | 'drop'
  | 'conflict'
  | 'possible_fix'
  | 'context'
  | 'unknown';

export interface OwnershipEvent {
  id: string;
  diagnosticId: string;
  kind: OwnershipEventKind;
  role: DiagnosticSpanRole;
  place?: string;
  spanId: string;
  message: string;
  evidence: Evidence[];
  confidence: Confidence;
}

export interface DiagnosticChildRecord {
  code?: string | null;
  level?: string;
  message: string;
  spans: DiagnosticSpan[];
  children?: DiagnosticChildRecord[];
  rendered?: string | null;
}

export interface DiagnosticRecord {
  id: string;
  code?: string | null;
  supported: boolean;
  level?: string;
  message: string;
  spans: DiagnosticSpan[];
  children?: DiagnosticChildRecord[];
  events?: OwnershipEvent[];
  learnerSummary?: LearnerSummary;
  fixStrategies?: FixStrategy[];
  rendered?: string | null;
  unsupportedReason?: string;
}

export interface DiagnosticReportSummary {
  totalDiagnostics: number;
  supportedDiagnostics: number;
  unsupportedDiagnostics: number;
  recommendedFirstFixes?: RecommendedFirstFix[];
  [key: string]: unknown;
}

export interface DiagnosticReportInput {
  path: string;
  commandFamily?: 'cargo-check-jsonl';
  audienceMode?: AudienceMode;
  rustcVersion?: string | null;
  cargoVersion?: string | null;
  [key: string]: unknown;
}

export interface DiagnosticReport {
  schemaVersion: '0.1.0';
  input: DiagnosticReportInput;
  summary: DiagnosticReportSummary;
  diagnostics: DiagnosticRecord[];
}
