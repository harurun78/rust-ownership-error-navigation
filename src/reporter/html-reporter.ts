import type {
  DiagnosticRecord,
  DiagnosticReport,
  OwnershipEvent
} from '../mapper/ownership-event.js';
import { escapeHtml, stableId } from './reporter-utils.js';

export function renderHtmlReport(report: DiagnosticReport): string {
  return `<!doctype html>
<html lang="en">
<head>
	<meta charset="utf-8">
	<title>Rust Ownership Diagnostic Report</title>
	<style>
		body { font-family: system-ui, sans-serif; margin: 2rem; line-height: 1.5; }
		table { border-collapse: collapse; width: 100%; margin: 1rem 0 2rem; }
		th, td { border: 1px solid #d0d7de; padding: 0.5rem; text-align: left; vertical-align: top; }
		th { background: #f6f8fa; }
		code, pre { background: #f6f8fa; padding: 0.125rem 0.25rem; }
		pre { overflow-x: auto; padding: 0.75rem; }
	</style>
</head>
<body>
	<h1>Rust Ownership Diagnostic Report</h1>
	${renderSummary(report)}
	${renderDiagnosticsOverview(report.diagnostics)}
	${renderCausalityTimeline(report.diagnostics)}
	${renderSourceSpans(report.diagnostics)}
	${renderEvidence(report.diagnostics)}
</body>
</html>
`;
}

function renderDiagnosticsOverview(diagnostics: readonly DiagnosticRecord[]): string {
  const rows = diagnostics.map(
    (diagnostic) =>
      `<tr><td>${escapeHtml(diagnostic.id)}</td><td>${escapeHtml(diagnostic.code ?? 'unknown')}</td><td>${escapeHtml(diagnostic.message)}</td><td>${diagnostic.supported ? 'supported' : 'unsupported'}</td></tr>`
  );

  return `<section id="diagnostics">
	<h2>Diagnostics</h2>
	<table>
		<thead><tr><th>ID</th><th>Code</th><th>Message</th><th>Status</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="4">No diagnostics</td></tr>'}</tbody>
	</table>
</section>`;
}

function renderSummary(report: DiagnosticReport): string {
  return `<section id="summary">
	<h2>Summary</h2>
	<table>
		<tbody>
			<tr><th>Input</th><td>${escapeHtml(report.input.path)}</td></tr>
			<tr><th>Total Diagnostics</th><td>${report.summary.totalDiagnostics}</td></tr>
			<tr><th>Supported Diagnostics</th><td>${report.summary.supportedDiagnostics}</td></tr>
			<tr><th>Unsupported Diagnostics</th><td>${report.summary.unsupportedDiagnostics}</td></tr>
		</tbody>
	</table>
</section>`;
}

function renderCausalityTimeline(diagnostics: readonly DiagnosticRecord[]): string {
  const rows = diagnostics.flatMap((diagnostic) =>
    (diagnostic.events ?? []).map(
      (event) =>
        `<tr id="${stableId('event', [event.id])}"><td>${escapeHtml(diagnostic.code ?? 'unknown')}</td><td>${escapeHtml(event.role)}</td><td>${escapeHtml(event.kind)}</td><td>${escapeHtml(event.message)}</td><td>${escapeHtml(event.spanId)}</td></tr>`
    )
  );

  return `<section id="causality-timeline">
	<h2>Causality Timeline</h2>
	<table>
		<thead><tr><th>Code</th><th>Role</th><th>Kind</th><th>Message</th><th>Span</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="5">No events</td></tr>'}</tbody>
	</table>
</section>`;
}

function renderSourceSpans(diagnostics: readonly DiagnosticRecord[]): string {
  const rows = diagnostics.flatMap((diagnostic) =>
    diagnostic.spans.map(
      (span) =>
        `<tr><td>${escapeHtml(diagnostic.id)}</td><td>${escapeHtml(span.role)}</td><td>${escapeHtml(span.file)}:${span.lineStart}:${span.columnStart}</td><td>${escapeHtml(span.label ?? '')}</td><td><pre>${escapeHtml(span.snippet ?? '')}</pre></td></tr>`
    )
  );

  return `<section id="source-spans">
	<h2>Source Spans</h2>
	<table>
		<thead><tr><th>Diagnostic</th><th>Role</th><th>Location</th><th>Label</th><th>Snippet</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="5">No spans</td></tr>'}</tbody>
	</table>
</section>`;
}

function renderEvidence(diagnostics: readonly DiagnosticRecord[]): string {
  const events = diagnostics.flatMap((diagnostic) => diagnostic.events ?? []);
  const rows = events.flatMap((event) => renderEvidenceRows(event));

  return `<section id="evidence">
	<h2>Evidence</h2>
	<table>
		<thead><tr><th>Event</th><th>Source</th><th>Field</th><th>Value</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="4">No evidence</td></tr>'}</tbody>
	</table>
</section>`;
}

function renderEvidenceRows(event: OwnershipEvent): string[] {
  return event.evidence.map(
    (evidence) =>
      `<tr><td>${escapeHtml(event.kind)}</td><td>${escapeHtml(evidence.source)}</td><td>${escapeHtml(evidence.field)}</td><td>${escapeHtml(String(evidence.value ?? ''))}</td></tr>`
  );
}
