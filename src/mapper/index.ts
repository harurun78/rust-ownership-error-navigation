import type { AudienceMode, DiagnosticRecord } from './ownership-event.js';
import { mapE0004Diagnostic } from './e0004.js';
import { mapE0308Diagnostic } from './e0308.js';
import { mapE0382Diagnostic } from './e0382.js';
import { mapE0425Diagnostic } from './e0425.js';
import { mapE0499Diagnostic } from './e0499.js';
import { mapE0502Diagnostic } from './e0502.js';
import { attachLearnerSummary } from './learner-summary.js';
import { createUnsupportedDiagnosticRecord } from './unsupported.js';

export const SUPPORTED_DIAGNOSTIC_CODES = [
  'E0382',
  'E0499',
  'E0502',
  'E0308',
  'E0004',
  'E0425'
] as const;

export type SupportedDiagnosticCode = (typeof SUPPORTED_DIAGNOSTIC_CODES)[number];

export type DiagnosticMapper = (diagnostic: DiagnosticRecord) => DiagnosticRecord;

export type MapperRegistry = Record<SupportedDiagnosticCode, DiagnosticMapper>;

export interface MapDiagnosticOptions {
  audienceMode?: AudienceMode;
}

export function isSupportedDiagnosticCode(
  code: string | null | undefined
): code is SupportedDiagnosticCode {
  return SUPPORTED_DIAGNOSTIC_CODES.includes(code as SupportedDiagnosticCode);
}

export const defaultMapperRegistry: MapperRegistry = {
  E0382: mapE0382Diagnostic,
  E0499: mapE0499Diagnostic,
  E0502: mapE0502Diagnostic,
  E0308: mapE0308Diagnostic,
  E0004: mapE0004Diagnostic,
  E0425: mapE0425Diagnostic
};

export function mapDiagnostic(
  diagnostic: DiagnosticRecord,
  registry: MapperRegistry = defaultMapperRegistry,
  options: MapDiagnosticOptions = {}
): DiagnosticRecord {
  if (!isSupportedDiagnosticCode(diagnostic.code)) {
    return createUnsupportedDiagnosticRecord(diagnostic);
  }

  return attachLearnerSummary(registry[diagnostic.code](diagnostic), options.audienceMode);
}

export function mapDiagnostics(
  diagnostics: readonly DiagnosticRecord[],
  registry: MapperRegistry = defaultMapperRegistry,
  options: MapDiagnosticOptions = {}
): DiagnosticRecord[] {
  return diagnostics.map((diagnostic) => mapDiagnostic(diagnostic, registry, options));
}
