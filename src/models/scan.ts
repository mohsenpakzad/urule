export interface ScanInfo {
  typ: ScanType;
  value: ScanValue;
}

export enum ScanType {
  Exact = 'Exact',
  Unknown = 'Unknown',
  InRange = 'InRange',
  Unchanged = 'Unchanged',
  Changed = 'Changed',
  Decreased = 'Decreased',
  Increased = 'Increased',
  DecreasedBy = 'DecreasedBy',
  IncreasedBy = 'IncreasedBy',
}

export interface ScanValue {
  Exact?: string;
  Range?: { start: string; end: string };
}

export enum ScanState {
  BeforeInitialScan = 1 << 0,
  AfterInitialScan = 1 << 1,
}

export enum ValueType {
  I8 = 'I8',
  U8 = 'U8',
  I16 = 'I16',
  U16 = 'U16',
  I32 = 'I32',
  U32 = 'U32',
  I64 = 'I64',
  U64 = 'U64',
  F32 = 'F32',
  F64 = 'F64',
}
