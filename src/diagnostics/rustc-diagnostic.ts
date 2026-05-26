export interface CargoMessage {
  reason: string;
  package_id?: string;
  manifest_path?: string;
  target?: unknown;
  message?: RustcDiagnostic;
  [key: string]: unknown;
}

export interface RustcDiagnosticCode {
  code: string;
  explanation?: string | null;
}

export interface RustcDiagnostic {
  code: RustcDiagnosticCode | null;
  level: string;
  message: string;
  spans: RustcSpan[];
  children: RustcChildDiagnostic[];
  rendered?: string | null;
  $message_type?: string;
}

export type RustcChildDiagnostic = RustcDiagnostic;

export interface RustcSpanText {
  text: string;
  highlight_start: number;
  highlight_end: number;
}

export interface RustcSpan {
  file_name: string;
  byte_start: number;
  byte_end: number;
  line_start: number;
  line_end: number;
  column_start: number;
  column_end: number;
  is_primary: boolean;
  label: string | null;
  text: RustcSpanText[];
  suggested_replacement?: string | null;
  suggestion_applicability?: string | null;
  expansion?: RustcSpanExpansion | null;
}

export interface RustcSpanExpansion {
  span?: RustcSpan;
  def_site_span?: RustcSpan;
  macro_decl_name?: string;
  [key: string]: unknown;
}
