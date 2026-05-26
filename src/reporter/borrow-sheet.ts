import type { Confidence, DiagnosticSpanRole } from '../diagnostics/diagnostic-span.js';
import type { DiagnosticRecord, OwnershipEventKind } from '../mapper/ownership-event.js';

export interface BorrowSheetRow {
  diagnosticId: string;
  diagnosticCode?: string | null;
  eventId: string;
  kind: OwnershipEventKind;
  role: DiagnosticSpanRole;
  place?: string;
  message: string;
  spanId: string;
  confidence: Confidence;
}

export function createBorrowSheetRows(diagnostics: readonly DiagnosticRecord[]): BorrowSheetRow[] {
  return diagnostics.flatMap((diagnostic) =>
    (diagnostic.events ?? []).map((event) => ({
      diagnosticId: diagnostic.id,
      diagnosticCode: diagnostic.code,
      eventId: event.id,
      kind: event.kind,
      role: event.role,
      place: event.place,
      message: event.message,
      spanId: event.spanId,
      confidence: event.confidence
    }))
  );
}
