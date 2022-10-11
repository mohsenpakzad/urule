export interface Range {
  start: number;
  end: number;
}

export interface Process {
  pid: number;
  name: string;
}

export interface CandidateLocations {
  KeyValue?: Record<number, number>;
  SameValue?: { locations: number[]; value: number };
  Range?: { range: Range; step: number; values: number[] };
  Offsetted?: { base: number; offsets: number[]; values: number[] };
  Masked?: { base: number; step: number; mask: boolean[]; values: number[] };
}

export interface Address {
  pointer: number;
  value: number;
}
