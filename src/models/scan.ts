export enum ScanType {
  Exact,
  Unknown,
  InRange,
  Unchanged,
  Changed,
  Decreased,
  Increased,
  DecreasedBy,
  IncreasedBy,
}

export enum ScanState {
  BeforeInitialScan = 1 << 0,
  AfterInitialScan = 1 << 1,
}

export enum ValueType {
  I8,
  U8,
  I16,
  U16,
  I32,
  U32,
  I64,
  U64,
  F32,
  F64,
}
