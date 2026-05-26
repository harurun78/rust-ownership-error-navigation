import type { DiagnosticRecord } from './ownership-event.js';
import { mapE0382Diagnostic } from './e0382.js';
import { mapE0499Diagnostic } from './e0499.js';
import { mapE0502Diagnostic } from './e0502.js';

export const SUPPORTED_DIAGNOSTIC_CODES = ['E0382', 'E0499', 'E0502'] as const;

export type SupportedDiagnosticCode = (typeof SUPPORTED_DIAGNOSTIC_CODES)[number];

export type DiagnosticMapper = (diagnostic: DiagnosticRecord) => DiagnosticRecord;

export type MapperRegistry = Record<SupportedDiagnosticCode, DiagnosticMapper>;

export function isSupportedDiagnosticCode(
  code: string | null | undefined
): code is SupportedDiagnosticCode {
  return SUPPORTED_DIAGNOSTIC_CODES.includes(code as SupportedDiagnosticCode);
}

export const defaultMapperRegistry: MapperRegistry = {
  E0382: mapE0382Diagnostic,
  E0499: mapE0499Diagnostic,
  E0502: mapE0502Diagnostic
};

export function mapDiagnostic(
  diagnostic: DiagnosticRecord,
  registry: MapperRegistry = defaultMapperRegistry
): DiagnosticRecord {
  if (!isSupportedDiagnosticCode(diagnostic.code)) {
    return createUnsupportedDiagnosticRecord(diagnostic);
  }

  return registry[diagnostic.code](diagnostic);
}

export function mapDiagnostics(
  diagnostics: readonly DiagnosticRecord[],
  registry: MapperRegistry = defaultMapperRegistry
): DiagnosticRecord[] {
  return diagnostics.map((diagnostic) => mapDiagnostic(diagnostic, registry));
}

export function createUnsupportedDiagnosticRecord(diagnostic: DiagnosticRecord): DiagnosticRecord {
  return {
    ...diagnostic,
    supported: false,
    events: diagnostic.events ?? [],
    unsupportedReason: diagnostic.unsupportedReason ?? unsupportedReasonForCode(diagnostic.code)
  };
}

function unsupportedReasonForCode(code: string | null | undefined): string {
  return code === null || code === undefined
    ? 'Diagnostic does not include a rustc error code.'
    : `Diagnostic code ${code} is outside the Phase 1 ownership mapping scope.`;
}
