import { createHash } from 'node:crypto';

import type { DiagnosticRecord, DiagnosticReportSummary } from '../mapper/ownership-event.js';

export const OWNERSHIP_DIAGNOSTIC_CODES = ['E0382', 'E0499', 'E0502'] as const;
export const NON_OWNERSHIP_DIAGNOSTIC_CODES = ['E0308', 'E0004', 'E0425'] as const;

const HTML_ESCAPE_PATTERN = /[&<>"']/g;

const HTML_ESCAPE_REPLACEMENTS: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;'
};

export function escapeHtml(value: string): string {
  return value.replace(
    HTML_ESCAPE_PATTERN,
    (character) => HTML_ESCAPE_REPLACEMENTS[character] ?? character
  );
}

export function stableId(prefix: string, parts: readonly unknown[]): string {
  const safePrefix = prefix
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
  const hash = createHash('sha256').update(JSON.stringify(parts)).digest('hex').slice(0, 12);

  return `${safePrefix || 'id'}-${hash}`;
}

export function createDiagnosticReportSummary(
  diagnostics: readonly DiagnosticRecord[]
): DiagnosticReportSummary {
  const supportedDiagnostics = diagnostics.filter((diagnostic) => diagnostic.supported).length;
  const ownershipDiagnostics = diagnostics.filter((diagnostic) =>
    isOwnershipDiagnosticCode(diagnostic.code)
  ).length;
  const nonOwnershipDiagnostics = diagnostics.filter((diagnostic) =>
    isNonOwnershipDiagnosticCode(diagnostic.code)
  ).length;

  return {
    totalDiagnostics: diagnostics.length,
    supportedDiagnostics,
    unsupportedDiagnostics: diagnostics.length - supportedDiagnostics,
    ownershipDiagnostics,
    nonOwnershipDiagnostics
  };
}

export function isOwnershipDiagnosticCode(code: string | null | undefined): boolean {
  return OWNERSHIP_DIAGNOSTIC_CODES.includes(code as (typeof OWNERSHIP_DIAGNOSTIC_CODES)[number]);
}

export function isNonOwnershipDiagnosticCode(code: string | null | undefined): boolean {
  return NON_OWNERSHIP_DIAGNOSTIC_CODES.includes(
    code as (typeof NON_OWNERSHIP_DIAGNOSTIC_CODES)[number]
  );
}
