import type {
  DiagnosticRecord,
  DiagnosticReport,
  DiagnosticReportInput
} from '../mapper/ownership-event.js';
import { createDiagnosticReportSummary } from './reporter-utils.js';

export interface CreateDiagnosticReportOptions {
  input: DiagnosticReportInput;
  diagnostics: readonly DiagnosticRecord[];
}

export function createDiagnosticReport(options: CreateDiagnosticReportOptions): DiagnosticReport {
  const diagnostics = [...options.diagnostics];

  return {
    schemaVersion: '0.1.0',
    input: {
      commandFamily: 'cargo-check-jsonl',
      ...options.input
    },
    summary: createDiagnosticReportSummary(diagnostics),
    diagnostics
  };
}

export function renderJsonReport(report: DiagnosticReport): string {
  return `${JSON.stringify(report, null, 2)}\n`;
}
