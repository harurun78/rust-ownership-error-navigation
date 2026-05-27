import type {
  DiagnosticRecord,
  DiagnosticReport,
  OwnershipEvent
} from '../mapper/ownership-event.js';
import { createBorrowSheetRows } from './borrow-sheet.js';
import {
  escapeHtml,
  isNonOwnershipDiagnosticCode,
  isOwnershipDiagnosticCode,
  stableId
} from './reporter-utils.js';

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
	${renderLearnerSummaries(report.diagnostics)}
	${renderDiagnosticGroups(report.diagnostics)}
	${renderDiagnosticsOverview(report.diagnostics)}
	${renderCausalityTimeline(report.diagnostics)}
	${renderSourceSpans(report.diagnostics)}
	${renderEvidence(report.diagnostics)}
	${renderBorrowSheet(report.diagnostics)}
	${renderUnsupportedDiagnostics(report.diagnostics)}
</body>
</html>
`;
}

function renderLearnerSummaries(diagnostics: readonly DiagnosticRecord[]): string {
  const summaryRows = diagnostics
    .filter((diagnostic) => diagnostic.learnerSummary !== undefined)
    .map((diagnostic) => {
      const summary = diagnostic.learnerSummary!;

      return `<article class="learner-summary-card" id="${stableId('learner-summary', [diagnostic.id])}">
		<h3>${escapeHtml(diagnostic.code ?? 'unknown')} learner summary</h3>
		<table>
			<tbody>
				<tr><th>Audience</th><td>${escapeHtml(summary.audience)}</td></tr>
				<tr><th>What Happened</th><td>${escapeHtml(summary.whatHappened)}</td></tr>
				<tr><th>Why It Matters</th><td>${escapeHtml(summary.whyItMatters)}</td></tr>
				<tr><th>Next Step</th><td>${escapeHtml(summary.nextStep)}</td></tr>
				<tr><th>Concepts</th><td>${escapeHtml(summary.conceptTerms?.join(', ') ?? '')}</td></tr>
				<tr><th>Confidence</th><td>${escapeHtml(summary.confidence)}</td></tr>
			</tbody>
		</table>
	</article>`;
    });

  return `<section id="learner-summaries">
	<h2>Learner Summaries</h2>
	${summaryRows.join('\n') || '<p>No learner summaries</p>'}
</section>`;
}

function renderDiagnosticGroups(diagnostics: readonly DiagnosticRecord[]): string {
  const groups = [
    {
      label: 'Ownership',
      diagnostics: diagnostics.filter((diagnostic) => isOwnershipDiagnosticCode(diagnostic.code))
    },
    {
      label: 'Non-Ownership',
      diagnostics: diagnostics.filter((diagnostic) => isNonOwnershipDiagnosticCode(diagnostic.code))
    },
    {
      label: 'Unsupported',
      diagnostics: diagnostics.filter((diagnostic) => !diagnostic.supported)
    }
  ];
  const rows = groups.map(
    (group) =>
      `<tr><td>${escapeHtml(group.label)}</td><td>${group.diagnostics.length}</td><td>${escapeHtml(group.diagnostics.map((diagnostic) => diagnostic.code ?? 'unknown').join(', ') || 'none')}</td></tr>`
  );

  return `<section id="diagnostic-groups">
	<h2>Diagnostic Groups</h2>
	<table>
		<thead><tr><th>Group</th><th>Count</th><th>Codes</th></tr></thead>
		<tbody>${rows.join('\n')}</tbody>
	</table>
</section>`;
}

function renderDiagnosticsOverview(diagnostics: readonly DiagnosticRecord[]): string {
  const rows = diagnostics.map(
    (diagnostic) =>
      `<tr><td>${escapeHtml(diagnostic.id)}</td><td>${escapeHtml(diagnosticGroupLabel(diagnostic))}</td><td>${escapeHtml(diagnostic.code ?? 'unknown')}</td><td>${escapeHtml(diagnostic.message)}</td><td>${diagnostic.supported ? 'supported' : 'unsupported'}</td></tr>`
  );

  return `<section id="diagnostics">
	<h2>Diagnostics</h2>
	<table>
		<thead><tr><th>ID</th><th>Group</th><th>Code</th><th>Message</th><th>Status</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="5">No diagnostics</td></tr>'}</tbody>
	</table>
</section>`;
}

function diagnosticGroupLabel(diagnostic: DiagnosticRecord): string {
  if (isOwnershipDiagnosticCode(diagnostic.code)) {
    return 'ownership';
  }

  if (isNonOwnershipDiagnosticCode(diagnostic.code)) {
    return 'non-ownership';
  }

  return diagnostic.supported ? 'supported' : 'unsupported';
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
    orderEvents(diagnostic.events ?? []).map(
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

function orderEvents(events: readonly OwnershipEvent[]): OwnershipEvent[] {
  return [...events].sort((left, right) => rolePriority(left.role) - rolePriority(right.role));
}

function rolePriority(role: string): number {
  switch (role) {
    case 'cause':
      return 1;
    case 'conflict':
      return 2;
    case 'context':
      return 3;
    case 'possible_fix':
      return 4;
    default:
      return 99;
  }
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

function renderUnsupportedDiagnostics(diagnostics: readonly DiagnosticRecord[]): string {
  const unsupportedDiagnostics = diagnostics.filter((diagnostic) => !diagnostic.supported);
  const rows = unsupportedDiagnostics.map(
    (diagnostic) =>
      `<tr><td>${escapeHtml(diagnostic.code ?? 'unknown')}</td><td>${escapeHtml(diagnostic.message)}</td><td>${escapeHtml(diagnostic.unsupportedReason ?? '')}</td><td><pre>${escapeHtml(diagnostic.rendered ?? '')}</pre></td></tr>`
  );

  return `<section id="unsupported-diagnostics">
	<h2>Unsupported Diagnostics</h2>
	<table>
		<thead><tr><th>Code</th><th>Message</th><th>Reason</th><th>Rendered</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="4">No unsupported diagnostics</td></tr>'}</tbody>
	</table>
</section>`;
}

function renderBorrowSheet(diagnostics: readonly DiagnosticRecord[]): string {
  const rows = createBorrowSheetRows(diagnostics).map(
    (row) =>
      `<tr><td>${escapeHtml(row.diagnosticCode ?? 'unknown')}</td><td>${escapeHtml(row.kind)}</td><td>${escapeHtml(row.role)}</td><td>${escapeHtml(row.place ?? '')}</td><td>${escapeHtml(row.message)}</td><td>${escapeHtml(row.spanId)}</td><td>${escapeHtml(row.confidence)}</td></tr>`
  );

  return `<section id="borrow-sheet">
	<h2>Borrow Sheet</h2>
	<table>
		<thead><tr><th>Code</th><th>Kind</th><th>Role</th><th>Place</th><th>Message</th><th>Span</th><th>Confidence</th></tr></thead>
		<tbody>${rows.join('\n') || '<tr><td colspan="7">No borrow sheet rows</td></tr>'}</tbody>
	</table>
</section>`;
}
