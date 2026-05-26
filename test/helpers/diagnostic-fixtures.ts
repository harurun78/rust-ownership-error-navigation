import { join } from 'node:path';

import { parseCargoMessagesFile } from '../../src/parser/cargo-message-parser.js';

export function diagnosticFixturePath(fileName: string): string {
  return join(process.cwd(), 'test', 'fixtures', 'diagnostics', fileName);
}

export function loadDiagnosticFixture(fileName: string) {
  return parseCargoMessagesFile(diagnosticFixturePath(fileName));
}
