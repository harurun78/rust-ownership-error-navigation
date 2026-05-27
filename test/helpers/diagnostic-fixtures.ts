import { join } from 'node:path';

import { normalizeRustcDiagnostic } from '../../src/diagnostics/normalizer.js';
import { mapDiagnostics } from '../../src/mapper/index.js';
import type { AudienceMode, DiagnosticReport } from '../../src/mapper/ownership-event.js';
import { parseCargoMessagesFile } from '../../src/parser/cargo-message-parser.js';
import { createDiagnosticReport } from '../../src/reporter/json-reporter.js';

export function diagnosticFixturePath(fileName: string): string {
  return join(process.cwd(), 'test', 'fixtures', 'diagnostics', fileName);
}

export function loadDiagnosticFixture(fileName: string) {
  return parseCargoMessagesFile(diagnosticFixturePath(fileName));
}

export async function createReportFromDiagnosticFixture(
  fileName: string,
  options: { audienceMode?: AudienceMode } = {}
): Promise<DiagnosticReport> {
  const cargoMessages = await loadDiagnosticFixture(fileName);
  const diagnostics = mapDiagnostics(
    cargoMessages.map((message, index) =>
      normalizeRustcDiagnostic(message.message!, { diagnosticIndex: index })
    )
  );

  return createDiagnosticReport({
    input: {
      path: fileName,
      ...(options.audienceMode === undefined ? {} : { audienceMode: options.audienceMode })
    },
    diagnostics
  });
}
