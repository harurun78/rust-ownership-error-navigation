import { describe, expect, it } from 'vitest';

import {
  CargoMessageParseError,
  parseCargoMessagesJsonl
} from '../../src/parser/cargo-message-parser.js';

describe('malformed JSONL parser errors', () => {
  it('reports the malformed line number', () => {
    expect(() => parseCargoMessagesJsonl('{"reason":"compiler-artifact"}\n{bad}\n{}')).toThrow(
      /line 2/
    );
  });

  it('throws a structured parser error for missing compiler diagnostic payloads', () => {
    try {
      parseCargoMessagesJsonl('{"reason":"compiler-message"}');
    } catch (error) {
      expect(error).toBeInstanceOf(CargoMessageParseError);
      expect((error as CargoMessageParseError).lineNumber).toBe(1);
      expect((error as CargoMessageParseError).lineText).toContain('compiler-message');
    }
  });
});
