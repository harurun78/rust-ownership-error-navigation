#!/usr/bin/env node

import process from 'node:process';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { normalizeRustcDiagnostic } from '../diagnostics/normalizer.js';
import type { AudienceMode } from '../mapper/diagnostic-navigation.js';
import { mapDiagnostics } from '../mapper/index.js';
import { parseCargoMessagesFile } from '../parser/cargo-message-parser.js';
import { createDiagnosticReport, renderJsonReport } from '../reporter/json-reporter.js';
import { renderHtmlReport } from '../reporter/html-reporter.js';

const HELP_TEXT = `rust-ownership-report

Usage:
  rust-ownership-report --input <diagnostics.jsonl> --json-out <report.json> --html-out <report.html> [--audience beginner|intermediate|agent]

Options:
  --input     Cargo/rustc JSONL diagnostics input file
  --json-out  JSON report output path
  --html-out  Static HTML report output path
  --audience  Report audience mode: beginner, intermediate, or agent (default: beginner)
  -h, --help  Show this help message`;

export interface CliOptions {
  input: string;
  jsonOut: string;
  htmlOut: string;
  audience: AudienceMode;
}

export async function main(argv = process.argv.slice(2)): Promise<number> {
  if (argv.includes('--help') || argv.includes('-h')) {
    console.log(HELP_TEXT);
    return 0;
  }

  try {
    const options = parseCliOptions(argv);
    const cargoMessages = await parseCargoMessagesFile(options.input);
    const normalizedDiagnostics = cargoMessages
      .map((message) => message.message)
      .filter((message) => message !== undefined)
      .map((diagnostic, index) => normalizeRustcDiagnostic(diagnostic, { diagnosticIndex: index }));
    const diagnostics = mapDiagnostics(normalizedDiagnostics, undefined, {
      audienceMode: options.audience
    });
    const report = createDiagnosticReport({
      input: { path: options.input, audienceMode: options.audience },
      diagnostics
    });

    await writeOutputFile(options.jsonOut, renderJsonReport(report));
    await writeOutputFile(options.htmlOut, renderHtmlReport(report));

    return 0;
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    return 1;
  }
}

export function parseCliOptions(argv: readonly string[]): CliOptions {
  const input = readOption(argv, '--input');
  const jsonOut = readOption(argv, '--json-out');
  const htmlOut = readOption(argv, '--html-out');
  const audience = parseAudienceMode(readOption(argv, '--audience'));

  if (input === undefined || jsonOut === undefined || htmlOut === undefined) {
    throw new Error(
      'Missing required arguments: --input, --json-out, and --html-out are required.'
    );
  }

  return { input, jsonOut, htmlOut, audience };
}

function parseAudienceMode(value: string | undefined): AudienceMode {
  if (value === undefined) {
    return 'beginner';
  }

  if (value === 'beginner' || value === 'intermediate' || value === 'agent') {
    return value;
  }

  throw new Error('Invalid value for --audience: expected beginner, intermediate, or agent.');
}

function readOption(argv: readonly string[], name: string): string | undefined {
  const index = argv.indexOf(name);
  if (index === -1) {
    return undefined;
  }

  const value = argv[index + 1];
  if (value === undefined || value.startsWith('--')) {
    throw new Error(`Missing value for ${name}.`);
  }

  return value;
}

async function writeOutputFile(filePath: string, contents: string): Promise<void> {
  await mkdir(dirname(filePath), { recursive: true });
  await writeFile(filePath, contents, 'utf8');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = await main();
}
