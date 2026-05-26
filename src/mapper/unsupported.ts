import type { DiagnosticRecord } from './ownership-event.js';

export function createUnsupportedDiagnosticRecord(diagnostic: DiagnosticRecord): DiagnosticRecord {
  return {
    ...diagnostic,
    supported: false,
    events: diagnostic.events ?? [],
    unsupportedReason: diagnostic.unsupportedReason ?? unsupportedReasonForCode(diagnostic.code)
  };
}

export function unsupportedReasonForCode(code: string | null | undefined): string {
  return code === null || code === undefined
    ? 'Diagnostic does not include a rustc error code.'
    : `Diagnostic code ${code} is outside the Phase 1 ownership mapping scope.`;
}
