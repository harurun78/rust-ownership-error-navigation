import type { Confidence, Evidence } from '../diagnostics/diagnostic-span.js';

export type AudienceMode = 'beginner' | 'intermediate' | 'agent';

export interface LearnerSummary {
  audience: AudienceMode;
  whatHappened: string;
  whyItMatters: string;
  nextStep: string;
  conceptTerms?: string[];
  evidence: Evidence[];
  confidence: Confidence;
}

export type FixStrategyKind =
  | 'borrow'
  | 'clone_or_copy'
  | 'split_scope'
  | 'move_timing'
  | 'extract_value'
  | 'redesign_ownership'
  | 'add_match_arm'
  | 'align_types'
  | 'resolve_name'
  | 'unknown';

export interface FixStrategy {
  id: string;
  diagnosticId: string;
  kind: FixStrategyKind;
  title: string;
  rationale: string;
  tradeOffs: string[];
  spanId?: string;
  evidence: Evidence[];
  confidence: Confidence;
}

export interface RecommendedFirstFix {
  diagnosticId: string;
  code?: string | null;
  priority: number;
  reason: string;
  nextStep: string;
  evidence: Evidence[];
  confidence: Confidence;
}
