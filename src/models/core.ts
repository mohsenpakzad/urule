export interface Range {
  start: number,
  end: number
}

export interface Process {
  pid: number;
  name?: string;
}

export interface CandidateLocations {
  Discrete?: { locations: number[] }
  SmallDiscrete?: { base: number, offsets: number[] },
  Dense?: { range: Range, step: number },
  Sparse?: { base: number, mask: boolean[], scale: number },
}

export interface Value {
  Exact?: number[],
  AnyWithin?: { memory: number[], size: number }
}

export interface Region {
  locations: CandidateLocations;
  value: Value;
}

export interface Address {
  name: string;
  value: number;
}
