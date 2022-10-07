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
  Range?: { range: Range; values: number[] };
  Offsetted?: { base: number; offsets: number[]; values: number[] };
  Masked?: { base: number; mask: boolean[]; values: number[] };
}
export interface Region {
  locations: CandidateLocations;
}

export interface Address {
  pointer: number;
  value: number;
}
