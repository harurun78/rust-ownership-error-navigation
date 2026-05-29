import { describe, expect, it } from 'vitest';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { attachDesignSuggestions } from '../../src/mapper/design-suggestion.js';
import { mapE0308Diagnostic } from '../../src/mapper/e0308.js';
import { mapE0499Diagnostic } from '../../src/mapper/e0499.js';
import { mapE0502Diagnostic } from '../../src/mapper/e0502.js';
import { mapDiagnostics } from '../../src/mapper/index.js';
import type { DiagnosticRecord } from '../../src/mapper/ownership-event.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('deterministic design suggestions', () => {
  it('adds split-mutation-phase to borrow conflicts with cause, conflict, and context', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const diagnostics = mapDiagnostics(
      messages.map((message, index) =>
        normalizeRustcDiagnostic(message.message!, { diagnosticIndex: index })
      ),
      undefined,
      { audienceMode: 'intermediate' }
    );
    const diagnostic = diagnostics.find((record) => record.code === 'E0502')!;

    expect(diagnostic.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'split-mutation-phase',
          confidence: 'high',
          evidence: expect.arrayContaining([
            expect.objectContaining({ source: 'heuristic', field: 'rule' })
          ])
        })
      ])
    );
  });

  it('adds avoid-long-lived-buffer-borrow when borrow conflict evidence mentions parser buffers', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const mapped = mapE0502Diagnostic(
      normalizeRustcDiagnostic(messages[2]!.message!, { diagnosticId: 'diagnostic-e0502-buffer' })
    );
    const diagnostic = attachDesignSuggestions({
      ...mapped,
      message: `${mapped.message} in parser output buffer`,
      spans: mapped.spans.map((span) => ({
        ...span,
        snippet: span.snippet === undefined ? 'stream.next_out buffer' : span.snippet
      }))
    });

    expect(diagnostic.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'avoid-long-lived-buffer-borrow',
          confidence: 'medium'
        })
      ])
    );
  });

  it('adds owned-result for moved value reuse and E0308 type boundary pressure', async () => {
    const ownershipMessages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');
    const ownershipDiagnostics = mapDiagnostics(
      ownershipMessages.map((message, index) =>
        normalizeRustcDiagnostic(message.message!, { diagnosticIndex: index })
      )
    );
    const e0382 = ownershipDiagnostics.find((record) => record.code === 'E0382')!;
    const portingMessages = await loadDiagnosticFixture(
      'porting/non-ownership-navigation-2026-05-25.jsonl'
    );
    const e0308Message = portingMessages.find((message) => message.message?.code?.code === 'E0308');
    const e0308 = attachDesignSuggestions(
      mapE0308Diagnostic(
        normalizeRustcDiagnostic(e0308Message!.message!, { diagnosticId: 'diagnostic-e0308' })
      )
    );

    expect(e0382.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: 'owned-result', confidence: 'high' })
      ])
    );
    expect(e0308.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: 'owned-result', confidence: 'medium' })
      ])
    );
  });

  it('adds arena and stable node ID suggestions for DOM-like E0499 tree pressure', () => {
    const diagnostic = attachDesignSuggestions(
      mapE0499Diagnostic(
        createBorrowConflictDiagnostic({
          code: 'E0499',
          message: 'cannot borrow `parent.children` as mutable more than once at a time',
          causeLabel: 'first mutable borrow occurs here',
          conflictLabel: 'second mutable borrow occurs here',
          causeSnippet: 'let child = Node::element(name, *parent);',
          conflictSnippet: 'parent.children.push(child);'
        })
      ),
      { audienceMode: 'intermediate' }
    );

    expect(diagnostic.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'arena-backed-tree',
          title: 'Use an arena to separate tree storage from mutation',
          evidence: expect.arrayContaining([
            expect.objectContaining({
              source: 'heuristic',
              field: 'rule',
              value: 'arena-backed-tree'
            }),
            expect.objectContaining({
              source: 'heuristic',
              field: 'trigger',
              value: 'parent.children'
            })
          ])
        }),
        expect.objectContaining({
          kind: 'stable-node-id',
          title: 'Represent parent and child links with stable IDs',
          evidence: expect.arrayContaining([
            expect.objectContaining({
              source: 'heuristic',
              field: 'rule',
              value: 'stable-node-id'
            }),
            expect.objectContaining({
              source: 'heuristic',
              field: 'trigger',
              value: 'parent.children'
            })
          ])
        })
      ])
    );
  });

  it('adds arena and stable node ID suggestions for E0502 stack and tree pressure', () => {
    const diagnostic = attachDesignSuggestions(
      mapE0502Diagnostic(
        createBorrowConflictDiagnostic({
          code: 'E0502',
          message: 'cannot borrow `self.stack` as mutable because it is also borrowed as immutable',
          causeLabel: 'immutable borrow occurs here',
          conflictLabel: 'mutable borrow occurs here',
          causeSnippet: 'let current = self.stack.last().expect("root node is open");',
          conflictSnippet: 'self.nodes[current.0].children.push(child);'
        })
      )
    );

    expect(diagnostic.designSuggestions?.map((suggestion) => suggestion.kind)).toEqual(
      expect.arrayContaining(['arena-backed-tree', 'stable-node-id'])
    );
  });

  it('adds self-referential guidance to unsupported E0515 diagnostics without marking them supported', () => {
    const [diagnostic] = mapDiagnostics([
      createUnsupportedSelfReferentialDiagnostic({
        code: 'E0515',
        message: 'cannot return value referencing local variable `root`',
        primaryLabel: 'returns a value referencing data owned by the current function',
        primarySnippet: 'Self { root, stack: vec![root_ref] }',
        causeLabel: '`root` is borrowed here',
        causeSnippet: 'let root_ref = &mut root;'
      })
    ]);

    expect(diagnostic?.supported).toBe(false);
    expect(diagnostic?.unsupportedReason).toContain('outside the Phase 1 ownership mapping scope');
    expect(diagnostic?.events).toEqual([]);
    expect(diagnostic?.designSuggestions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'avoid-self-referential-struct',
          title: 'Avoid storing a reference into the same returned object',
          confidence: 'medium',
          evidence: expect.arrayContaining([
            expect.objectContaining({
              source: 'heuristic',
              field: 'rule',
              value: 'avoid-self-referential-struct'
            })
          ])
        })
      ])
    );
  });

  it('adds self-referential guidance to unsupported E0505 construction diagnostics', () => {
    const [diagnostic] = mapDiagnostics([
      createUnsupportedSelfReferentialDiagnostic({
        code: 'E0505',
        message: 'cannot move out of `root` because it is borrowed',
        primaryLabel: 'move out of `root` occurs here',
        primarySnippet: 'Self { root, stack: vec![root_ref] }',
        causeLabel: 'borrow of `root` occurs here',
        causeSnippet: 'let root_ref = &mut root;'
      })
    ]);

    expect(diagnostic?.supported).toBe(false);
    expect(diagnostic?.designSuggestions?.map((suggestion) => suggestion.kind)).toEqual([
      'avoid-self-referential-struct'
    ]);
  });

  it('does not add self-referential guidance to ordinary unsupported E0505 move-after-borrow diagnostics', () => {
    const [diagnostic] = mapDiagnostics([
      createUnsupportedSelfReferentialDiagnostic({
        code: 'E0505',
        message: 'cannot move out of `value` because it is borrowed',
        primaryLabel: 'move out of `value` occurs here',
        primarySnippet: 'consume(value);',
        causeLabel: 'borrow of `value` occurs here',
        causeSnippet: 'let borrowed = &value;'
      })
    ]);

    expect(diagnostic?.supported).toBe(false);
    expect(diagnostic?.designSuggestions).toBeUndefined();
  });
});

function createBorrowConflictDiagnostic(options: {
  code: 'E0499' | 'E0502';
  message: string;
  causeLabel: string;
  conflictLabel: string;
  causeSnippet: string;
  conflictSnippet: string;
}): DiagnosticRecord {
  return {
    id: `diagnostic-${options.code.toLowerCase()}-tree`,
    code: options.code,
    supported: true,
    level: 'error',
    message: options.message,
    spans: [
      {
        id: 'span-cause',
        diagnosticId: `diagnostic-${options.code.toLowerCase()}-tree`,
        role: 'unknown',
        file: 'src/lib.rs',
        lineStart: 10,
        lineEnd: 10,
        columnStart: 9,
        columnEnd: 40,
        byteStart: 0,
        byteEnd: 0,
        isPrimary: false,
        label: options.causeLabel,
        snippet: options.causeSnippet,
        suggestedReplacement: null,
        suggestionApplicability: null,
        hasExpansion: false,
        evidence: [
          { source: 'rustc_span_label', field: 'label', value: options.causeLabel },
          { source: 'rustc_span_text', field: 'text', value: options.causeSnippet }
        ],
        confidence: 'high'
      },
      {
        id: 'span-conflict',
        diagnosticId: `diagnostic-${options.code.toLowerCase()}-tree`,
        role: 'unknown',
        file: 'src/lib.rs',
        lineStart: 11,
        lineEnd: 11,
        columnStart: 9,
        columnEnd: 40,
        byteStart: 0,
        byteEnd: 0,
        isPrimary: true,
        label: options.conflictLabel,
        snippet: options.conflictSnippet,
        suggestedReplacement: null,
        suggestionApplicability: null,
        hasExpansion: false,
        evidence: [
          { source: 'rustc_primary_span', field: 'is_primary', value: true },
          { source: 'rustc_span_label', field: 'label', value: options.conflictLabel },
          { source: 'rustc_span_text', field: 'text', value: options.conflictSnippet }
        ],
        confidence: 'high'
      }
    ],
    children: []
  };
}

function createUnsupportedSelfReferentialDiagnostic(options: {
  code: 'E0505' | 'E0515';
  message: string;
  primaryLabel: string;
  primarySnippet: string;
  causeLabel: string;
  causeSnippet: string;
}): DiagnosticRecord {
  return {
    id: `diagnostic-${options.code.toLowerCase()}-self-referential`,
    code: options.code,
    supported: false,
    level: 'error',
    message: options.message,
    spans: [
      {
        id: 'span-self-ref-primary',
        diagnosticId: `diagnostic-${options.code.toLowerCase()}-self-referential`,
        role: 'unknown',
        file: 'src/lib.rs',
        lineStart: 55,
        lineEnd: 58,
        columnStart: 9,
        columnEnd: 10,
        byteStart: 0,
        byteEnd: 0,
        isPrimary: true,
        label: options.primaryLabel,
        snippet: options.primarySnippet,
        suggestedReplacement: null,
        suggestionApplicability: null,
        hasExpansion: false,
        evidence: [
          { source: 'rustc_primary_span', field: 'is_primary', value: true },
          { source: 'rustc_span_label', field: 'label', value: options.primaryLabel },
          { source: 'rustc_span_text', field: 'text', value: options.primarySnippet }
        ],
        confidence: 'high'
      },
      {
        id: 'span-self-ref-cause',
        diagnosticId: `diagnostic-${options.code.toLowerCase()}-self-referential`,
        role: 'unknown',
        file: 'src/lib.rs',
        lineStart: 54,
        lineEnd: 54,
        columnStart: 24,
        columnEnd: 33,
        byteStart: 0,
        byteEnd: 0,
        isPrimary: false,
        label: options.causeLabel,
        snippet: options.causeSnippet,
        suggestedReplacement: null,
        suggestionApplicability: null,
        hasExpansion: false,
        evidence: [
          { source: 'rustc_span_label', field: 'label', value: options.causeLabel },
          { source: 'rustc_span_text', field: 'text', value: options.causeSnippet }
        ],
        confidence: 'high'
      }
    ],
    children: []
  };
}
