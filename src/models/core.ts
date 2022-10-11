export interface Range {
  start: number;
  end: number;
}

export interface Process {
  pid: number;
  name: string;
}

export interface Location {
  address: number;
  value: number;
}
