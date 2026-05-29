import type { Evidence } from '../diagnostics/diagnostic-span.js';
import type { AudienceMode, DesignSuggestion, DiagnosticRecord } from './ownership-event.js';

export interface DeriveDesignSuggestionsOptions {
  audienceMode?: AudienceMode;
}

export function attachDesignSuggestions(
  diagnostic: DiagnosticRecord,
  options: DeriveDesignSuggestionsOptions = {}
): DiagnosticRecord {
  const designSuggestions = deriveDesignSuggestions(diagnostic, options);

  return designSuggestions.length === 0 ? diagnostic : { ...diagnostic, designSuggestions };
}

export function deriveDesignSuggestions(
  diagnostic: DiagnosticRecord,
  options: DeriveDesignSuggestionsOptions = {}
): DesignSuggestion[] {
  if (!diagnostic.supported) {
    return [];
  }

  const suggestions = [
    deriveSplitMutationPhase(diagnostic, options.audienceMode),
    deriveAvoidLongLivedBufferBorrow(diagnostic, options.audienceMode),
    deriveArenaBackedTree(diagnostic, options.audienceMode),
    deriveStableNodeId(diagnostic, options.audienceMode),
    deriveOwnedResult(diagnostic, options.audienceMode)
  ].filter((suggestion): suggestion is DesignSuggestion => suggestion !== undefined);

  return dedupeSuggestions(suggestions);
}

function deriveSplitMutationPhase(
  diagnostic: DiagnosticRecord,
  audienceMode: AudienceMode | undefined
): DesignSuggestion | undefined {
  if (diagnostic.code !== 'E0499' && diagnostic.code !== 'E0502') {
    return undefined;
  }

  const hasCause = diagnostic.events?.some((event) => event.role === 'cause') ?? false;
  const hasConflict = diagnostic.events?.some((event) => event.role === 'conflict') ?? false;
  const hasContext = diagnostic.events?.some((event) => event.role === 'context') ?? false;

  if (!hasCause || !hasConflict || !hasContext) {
    return undefined;
  }

  return {
    id: `${diagnostic.id}-design-suggestion-split-mutation-phase`,
    diagnosticId: diagnostic.id,
    kind: 'split-mutation-phase',
    title: titleForAudience(
      audienceMode,
      'Split reading and mutation into separate phases',
      'Split borrow phases',
      'Separate read phase from mutation phase'
    ),
    why: 'The diagnostic has an earlier borrow, a conflicting borrow or mutation, and a later use of the earlier borrow.',
    whenToUse:
      'Use this when the code can finish reading from the borrowed value before starting the mutation, or when an owned snapshot can cross the boundary.',
    caution:
      'Do not change observable ordering if the mutation must happen before the later read; use the source spans to confirm behavior first.',
    evidence: evidenceFor(diagnostic, 'split-mutation-phase', 'cause-conflict-context'),
    confidence: 'high'
  };
}

function deriveAvoidLongLivedBufferBorrow(
  diagnostic: DiagnosticRecord,
  audienceMode: AudienceMode | undefined
): DesignSuggestion | undefined {
  if (diagnostic.code !== 'E0499' && diagnostic.code !== 'E0502') {
    return undefined;
  }

  const matchedTerm = findBufferPressureTerm(diagnostic);
  if (matchedTerm === undefined) {
    return undefined;
  }

  return {
    id: `${diagnostic.id}-design-suggestion-avoid-long-lived-buffer-borrow`,
    diagnosticId: diagnostic.id,
    kind: 'avoid-long-lived-buffer-borrow',
    title: titleForAudience(
      audienceMode,
      'Avoid keeping parser buffers borrowed for too long',
      'Avoid long-lived buffer borrows',
      'Prefer owned parse output over stored buffer borrows'
    ),
    why: 'The diagnostic evidence mentions parser, stream, input, output, or buffer state while a borrow conflict is present.',
    whenToUse:
      'Use this when a C-style port stores references to caller input or output buffers across parser state transitions.',
    caution:
      'Owned records can allocate or copy more data; keep borrowed views only when the lifetime boundary is small and clear.',
    evidence: evidenceFor(diagnostic, 'avoid-long-lived-buffer-borrow', matchedTerm),
    confidence: 'medium'
  };
}

function deriveArenaBackedTree(
  diagnostic: DiagnosticRecord,
  audienceMode: AudienceMode | undefined
): DesignSuggestion | undefined {
  if (diagnostic.code !== 'E0499' && diagnostic.code !== 'E0502') {
    return undefined;
  }

  const matchedTerm = findTreePressureTerm(diagnostic);
  if (matchedTerm === undefined) {
    return undefined;
  }

  return {
    id: `${diagnostic.id}-design-suggestion-arena-backed-tree`,
    diagnosticId: diagnostic.id,
    kind: 'arena-backed-tree',
    title: titleForAudience(
      audienceMode,
      'Store tree nodes in an arena',
      'Use arena-backed tree storage',
      'Use an arena to separate tree storage from mutation'
    ),
    why: 'The diagnostic evidence mentions tree, node, parent, child, or stack state while mutable access overlaps.',
    whenToUse:
      'Use this when ported object-graph code stores parent and child references while also mutating the tree.',
    caution:
      'Arena storage changes identity from references to indexes; keep APIs explicit about when a node ID can be resolved back to a node.',
    evidence: evidenceFor(diagnostic, 'arena-backed-tree', matchedTerm),
    confidence: 'medium'
  };
}

function deriveStableNodeId(
  diagnostic: DiagnosticRecord,
  audienceMode: AudienceMode | undefined
): DesignSuggestion | undefined {
  if (diagnostic.code !== 'E0499' && diagnostic.code !== 'E0502') {
    return undefined;
  }

  const matchedTerm = findTreePressureTerm(diagnostic);
  if (matchedTerm === undefined) {
    return undefined;
  }

  return {
    id: `${diagnostic.id}-design-suggestion-stable-node-id`,
    diagnosticId: diagnostic.id,
    kind: 'stable-node-id',
    title: titleForAudience(
      audienceMode,
      'Link nodes with stable IDs instead of references',
      'Use stable node IDs for tree links',
      'Represent parent and child links with stable IDs'
    ),
    why: 'The diagnostic evidence suggests direct references are being used as both tree identity and mutation access paths.',
    whenToUse:
      'Use this when a stack, parent link, or child list needs to remember a node without holding a long-lived Rust reference.',
    caution:
      'IDs avoid borrow conflicts but require lookup validation and clear ownership of the backing arena.',
    evidence: evidenceFor(diagnostic, 'stable-node-id', matchedTerm),
    confidence: 'medium'
  };
}

function deriveOwnedResult(
  diagnostic: DiagnosticRecord,
  audienceMode: AudienceMode | undefined
): DesignSuggestion | undefined {
  if (diagnostic.code !== 'E0382' && diagnostic.code !== 'E0308') {
    return undefined;
  }

  const hasMovedValueReuse =
    diagnostic.code === 'E0382' &&
    (diagnostic.events?.some((event) => event.kind === 'move') ?? false) &&
    (diagnostic.events?.some((event) => event.kind === 'use') ?? false);
  const hasTypeBoundaryPressure =
    diagnostic.code === 'E0308' &&
    (textEvidence(diagnostic).includes('expected') || textEvidence(diagnostic).includes('found'));

  if (!hasMovedValueReuse && !hasTypeBoundaryPressure) {
    return undefined;
  }

  return {
    id: `${diagnostic.id}-design-suggestion-owned-result`,
    diagnosticId: diagnostic.id,
    kind: 'owned-result',
    title: titleForAudience(
      audienceMode,
      'Return owned values at the API boundary',
      'Use owned results at ownership boundaries',
      'Prefer owned return values over C-style output pressure'
    ),
    why: 'The diagnostic indicates value reuse after a move or a type boundary mismatch where ownership should be explicit.',
    whenToUse:
      'Use this when a function can return the produced value, parse record, or builder result instead of mutating an output slot or reusing a moved binding.',
    caution:
      'Returning owned values can move allocation or construction cost to the caller; preserve borrowing when the caller truly needs a view into existing data.',
    evidence: evidenceFor(diagnostic, 'owned-result', diagnostic.code ?? 'unknown'),
    confidence: diagnostic.code === 'E0382' ? 'high' : 'medium'
  };
}

function evidenceFor(diagnostic: DiagnosticRecord, rule: string, trigger: string): Evidence[] {
  const primarySpan = diagnostic.spans.find((span) => span.isPrimary) ?? diagnostic.spans[0];
  const spanLabel = diagnostic.spans.find(
    (span) => span.label !== null && span.label !== undefined
  );

  return [
    { source: 'diagnostic_code', field: 'code', value: diagnostic.code ?? null },
    { source: 'heuristic', field: 'rule', value: rule },
    { source: 'heuristic', field: 'trigger', value: trigger },
    ...(primarySpan === undefined
      ? []
      : [{ source: 'rustc_primary_span' as const, field: 'spanId', value: primarySpan.id }]),
    ...(spanLabel?.label === undefined || spanLabel.label === null
      ? []
      : [{ source: 'rustc_span_label' as const, field: 'label', value: spanLabel.label }])
  ];
}

function findBufferPressureTerm(diagnostic: DiagnosticRecord): string | undefined {
  const text = textEvidence(diagnostic);
  const terms = ['buffer', 'input', 'output', 'parser', 'stream', 'next_in', 'next_out'];

  return terms.find((term) => text.includes(term));
}

function findTreePressureTerm(diagnostic: DiagnosticRecord): string | undefined {
  const text = textEvidence(diagnostic);
  const explicitTerms = [
    'parent.children',
    'child list',
    'child-list',
    'open stack',
    'object graph',
    'object-graph',
    'nodeid',
    'node id'
  ];
  const explicitTerm = explicitTerms.find((term) => text.includes(term));
  if (explicitTerm !== undefined) {
    return explicitTerm;
  }

  const relationTerms = ['parent', 'child', 'children', 'stack'];
  const identityTerms = ['node', 'root', 'tree', 'dom', 'element'];
  const relationTerm = relationTerms.find((term) => text.includes(term));
  const identityTerm = identityTerms.find((term) => text.includes(term));

  return relationTerm !== undefined && identityTerm !== undefined
    ? `${relationTerm}+${identityTerm}`
    : undefined;
}

function textEvidence(diagnostic: DiagnosticRecord): string {
  return [
    diagnostic.message,
    ...diagnostic.spans.flatMap((span) => [span.label ?? '', span.snippet ?? '', span.file]),
    ...(diagnostic.events ?? []).map((event) => event.message),
    ...(diagnostic.children ?? []).flatMap((child) => [
      child.message,
      ...child.spans.flatMap((span) => [span.label ?? '', span.snippet ?? '', span.file])
    ])
  ]
    .join('\n')
    .toLowerCase();
}

function dedupeSuggestions(suggestions: readonly DesignSuggestion[]): DesignSuggestion[] {
  const seenKinds = new Set<string>();

  return suggestions.filter((suggestion) => {
    if (seenKinds.has(suggestion.kind)) {
      return false;
    }

    seenKinds.add(suggestion.kind);
    return true;
  });
}

function titleForAudience(
  audienceMode: AudienceMode | undefined,
  beginnerTitle: string,
  agentTitle: string,
  intermediateTitle: string
): string {
  if (audienceMode === 'agent') {
    return agentTitle;
  }

  if (audienceMode === 'intermediate') {
    return intermediateTitle;
  }

  return beginnerTitle;
}
