import { createHash } from 'node:crypto';

import type { DiagnosticRecord, DiagnosticReportSummary } from '../mapper/ownership-event.js';

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

  return {
    totalDiagnostics: diagnostics.length,
    supportedDiagnostics,
    unsupportedDiagnostics: diagnostics.length - supportedDiagnostics
  };
}
