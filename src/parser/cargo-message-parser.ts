import { readFile } from 'node:fs/promises';

import type { CargoMessage } from '../diagnostics/rustc-diagnostic.js';

export class CargoMessageParseError extends Error {
  readonly lineNumber: number;
  readonly lineText: string;

  constructor(lineNumber: number, lineText: string, cause: unknown) {
    super(`Malformed Cargo JSON message at line ${lineNumber}: ${formatCause(cause)}`);
    this.name = 'CargoMessageParseError';
    this.lineNumber = lineNumber;
    this.lineText = lineText;
    this.cause = cause;
  }
}

export function parseCargoMessageLine(line: string, lineNumber = 1): CargoMessage | null {
  if (line.trim().length === 0) {
    return null;
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(line);
  } catch (error) {
    throw new CargoMessageParseError(lineNumber, line, error);
  }

  if (!isRecord(parsed) || parsed.reason !== 'compiler-message') {
    return null;
  }

  if (!isRecord(parsed.message)) {
    throw new CargoMessageParseError(
      lineNumber,
      line,
      'compiler-message record is missing a diagnostic message payload'
    );
  }

  return parsed as CargoMessage;
}

export function parseCargoMessagesJsonl(contents: string): CargoMessage[] {
  const messages: CargoMessage[] = [];
  const lines = contents.split(/\r?\n/);

  for (const [index, line] of lines.entries()) {
    const message = parseCargoMessageLine(line, index + 1);
    if (message !== null) {
      messages.push(message);
    }
  }

  return messages;
}

export async function parseCargoMessagesFile(inputPath: string): Promise<CargoMessage[]> {
  const contents = await readFile(inputPath, 'utf8');
  return parseCargoMessagesJsonl(contents);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function formatCause(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
