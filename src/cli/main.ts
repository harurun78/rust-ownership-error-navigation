#!/usr/bin/env node

import process from 'node:process';
import { fileURLToPath } from 'node:url';

const HELP_TEXT = `rust-ownership-report

Usage:
  rust-ownership-report --input <diagnostics.jsonl> --json-out <report.json> --html-out <report.html>

Options:
  --input     Cargo/rustc JSONL diagnostics input file
  --json-out  JSON report output path
  --html-out  Static HTML report output path
  -h, --help  Show this help message`;

export function main(argv = process.argv.slice(2)): number {
  if (argv.includes('--help') || argv.includes('-h')) {
    console.log(HELP_TEXT);
    return 0;
  }

  console.error('rust-ownership-report CLI implementation is pending. Run with --help for usage.');
  return 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main();
}
