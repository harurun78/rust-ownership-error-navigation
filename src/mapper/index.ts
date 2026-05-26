import type { DiagnosticRecord } from './ownership-event.js';

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
  E0382: markSupportedDiagnostic,
  E0499: markSupportedDiagnostic,
  E0502: markSupportedDiagnostic
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

function markSupportedDiagnostic(diagnostic: DiagnosticRecord): DiagnosticRecord {
  return {
    ...diagnostic,
    supported: true,
    events: diagnostic.events ?? []
  };
}

function unsupportedReasonForCode(code: string | null | undefined): string {
  return code === null || code === undefined
    ? 'Diagnostic does not include a rustc error code.'
    : `Diagnostic code ${code} is outside the Phase 1 ownership mapping scope.`;
}
