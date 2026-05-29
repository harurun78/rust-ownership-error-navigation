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
    return dedupeSuggestions(
      [deriveAvoidSelfReferentialStruct(diagnostic, options.audienceMode)].filter(
        (suggestion): suggestion is DesignSuggestion => suggestion !== undefined
      )
    );
  }

  const suggestions = [
    deriveSplitMutationPhase(diagnostic, options.audienceMode),
    deriveAvoidLongLivedBufferBorrow(diagnostic, options.audienceMode),
    deriveArenaBackedTree(diagnostic, options.audienceMode),
    deriveStableNodeId(diagnostic, options.audienceMode),
    deriveAvoidSelfReferentialStruct(diagnostic, options.audienceMode),
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

function deriveAvoidSelfReferentialStruct(
  diagnostic: DiagnosticRecord,
  audienceMode: AudienceMode | undefined
): DesignSuggestion | undefined {
  if (diagnostic.code !== 'E0505' && diagnostic.code !== 'E0515') {
    return undefined;
  }

  const matchedTerm = findSelfReferentialPressureTerm(diagnostic);
  if (matchedTerm === undefined) {
    return undefined;
  }

  return {
    id: `${diagnostic.id}-design-suggestion-avoid-self-referential-struct`,
    diagnosticId: diagnostic.id,
    kind: 'avoid-self-referential-struct',
    title: titleForAudience(
      audienceMode,
      'Avoid storing a reference into the same returned object',
      'Avoid self-referential struct shape',
      'Avoid self-referential structs; use IDs or owned storage'
    ),
    why: 'The diagnostic evidence indicates a value is returned or moved while a reference to its local state is still required.',
    whenToUse:
      'Use this when a ported object graph tries to store references to nodes, fields, or locals inside the same returned structure.',
    caution:
      'This is guidance only for an unsupported diagnostic mapping; confirm behavior before replacing references with IDs, indexes, or owned values.',
    evidence: evidenceFor(diagnostic, 'avoid-self-referential-struct', matchedTerm),
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
  const typeBoundaryTrigger = findOwnedResultTypeBoundaryTerm(diagnostic);
  const hasTypeBoundaryPressure = diagnostic.code === 'E0308' && typeBoundaryTrigger !== undefined;

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
      diagnostic.code === 'E0308'
        ? 'Use this when a behavior-only Rust API can return Result or an owned value directly, or when compatibility code needs an explicit adapter from Result into an output slot.'
        : 'Use this when a function can return the produced value, parse record, or builder result instead of mutating an output slot or reusing a moved binding.',
    caution:
      diagnostic.code === 'E0308'
        ? 'Do not imply a strict compatibility-preserving API can always change shape; keep a C-shaped boundary when required and adapt internally.'
        : 'Returning owned values can move allocation or construction cost to the caller; preserve borrowing when the caller truly needs a view into existing data.',
    evidence: evidenceFor(
      diagnostic,
      'owned-result',
      typeBoundaryTrigger ?? diagnostic.code ?? 'unknown'
    ),
    confidence: diagnostic.code === 'E0382' ? 'high' : 'medium'
  };
}

function findOwnedResultTypeBoundaryTerm(diagnostic: DiagnosticRecord): string | undefined {
  if (diagnostic.code !== 'E0308') {
    return undefined;
  }

  const text = textEvidence(diagnostic);
  const hasExpectedFound = text.includes('expected') && text.includes('found');
  if (!hasExpectedFound) {
    return undefined;
  }

  const boundaryTerms = [
    { trigger: 'result-option-boundary', terms: ['result<', 'option<'] },
    { trigger: 'result-option-boundary', terms: ['result<', 'option '] },
    { trigger: 'result-error-boundary', terms: ['result<', 'parseerror'] },
    { trigger: 'mutable-out-param', terms: ['&mut'] },
    { trigger: 'raw-output-pointer', terms: ['*mut'] },
    { trigger: 'raw-output-pointer', terms: ['*const'] },
    { trigger: 'out-param', terms: ['out-param'] },
    { trigger: 'out-param', terms: ['out parameter'] },
    { trigger: 'out-param', terms: ['out_parameter'] },
    { trigger: 'out-param', terms: ['error_out'] },
    { trigger: 'output-buffer', terms: ['next_out'] },
    { trigger: 'output-buffer', terms: ['output buffer'] },
    { trigger: 'output-buffer', terms: ['caller-visible output'] }
  ];

  return boundaryTerms.find((candidate) => candidate.terms.every((term) => text.includes(term)))
    ?.trigger;
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

function findSelfReferentialPressureTerm(diagnostic: DiagnosticRecord): string | undefined {
  const text = textEvidence(diagnostic);

  if (diagnostic.code === 'E0515') {
    const returnTerms = [
      'cannot return value referencing local variable',
      'returns a value referencing data owned by the current function',
      'returning this value requires',
      'local variable'
    ];

    return returnTerms.find((term) => text.includes(term));
  }

  const moveAfterBorrow = text.includes('cannot move out') && text.includes('borrow');
  if (!moveAfterBorrow) {
    return undefined;
  }

  const constructionTerms = [
    'returning this value requires',
    'self-referential',
    'self {',
    'struct {',
    'stack: vec!',
    'root_ref',
    'local variable'
  ];

  return constructionTerms.find((term) => text.includes(term));
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
