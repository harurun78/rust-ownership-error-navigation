import { describe, expect, it } from 'vitest';

import {
  CargoMessageParseError,
  parseCargoMessagesJsonl
} from '../../src/parser/cargo-message-parser.js';
import { loadDiagnosticFixture } from '../helpers/diagnostic-fixtures.js';

describe('cargo message parser', () => {
  it('extracts compiler-message diagnostics from fixture JSONL', async () => {
    const messages = await loadDiagnosticFixture('ownership-baseline-2026-05-24.jsonl');

    expect(messages).toHaveLength(5);
    expect(messages[0]?.message?.code?.code).toBe('E0382');
    expect(messages[1]?.message?.code?.code).toBe('E0499');
    expect(messages[2]?.message?.code?.code).toBe('E0502');
  });

  it('skips non-compiler-message records', () => {
    const compilerArtifact = JSON.stringify({ reason: 'compiler-artifact', target: {} });
    const compilerMessage = JSON.stringify({
      reason: 'compiler-message',
      message: {
        code: null,
        level: 'warning',
        message: 'unused variable',
        spans: [],
        children: []
      }
    });

    const messages = parseCargoMessagesJsonl(`${compilerArtifact}\n${compilerMessage}`);

    expect(messages).toHaveLength(1);
    expect(messages[0]?.message?.message).toBe('unused variable');
  });

  it('reports malformed JSONL with a line number', () => {
    const validLine = JSON.stringify({ reason: 'compiler-artifact' });

    expect(() => parseCargoMessagesJsonl(`${validLine}\n{not-json}`)).toThrow(
      CargoMessageParseError
    );
    expect(() => parseCargoMessagesJsonl(`${validLine}\n{not-json}`)).toThrow(/line 2/);
  });

  it('reports compiler-message records without diagnostic payloads as malformed', () => {
    const missingMessage = JSON.stringify({ reason: 'compiler-message' });

    expect(() => parseCargoMessagesJsonl(missingMessage)).toThrow(CargoMessageParseError);
    expect(() => parseCargoMessagesJsonl(missingMessage)).toThrow(/missing a diagnostic message/);
  });
});
