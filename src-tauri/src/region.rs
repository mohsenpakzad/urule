use serde::Serialize;
use std::ops::Range;
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// A memory region.
#[derive(Clone, Serialize)]
pub struct Region {
    /// The raw information about this memory region.
    #[serde(skip_serializing)]
    pub info: MEMORY_BASIC_INFORMATION,
    /// Candidate locations that should be considered during subsequent scans.
    pub locations: CandidateLocations,
    /// The value (or value range) to compare against during subsequent scans.
    pub value: Value,
}

unsafe impl Send for Region {}

impl Region {
    /// Return the value stored at `addr`.
    pub fn value_at(&self, addr: usize) -> &[u8] {
        match &self.value {
            Value::Exact(n) => n,
            Value::AnyWithin { memory, size } => {
                let base = addr - self.info.BaseAddress as usize;
                &memory[base..base + size]
            }
        }
    }
}

/// Candidate memory locations for holding our desired value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum CandidateLocations {
    /// Multiple, separated locations.
    ///
    /// It is a logic error to have the locations in non-ascending order.
    Discrete { locations: Vec<usize> },
    /// A dense memory location. Everything within here should be considered.
    Dense { range: Range<usize>, step: usize },
}

impl CandidateLocations {
    /// Return the amount of candidate locations.
    pub fn len(&self) -> usize {
        match self {
            CandidateLocations::Discrete { locations } => locations.len(),
            CandidateLocations::Dense { range, step } => range.len() / step,
        }
    }

    /// Return a iterator over the locations.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        match self {
            CandidateLocations::Discrete { locations } => Box::new(locations.iter().copied()),
            CandidateLocations::Dense { range, step } => Box::new(range.clone().step_by(*step)),
        }
    }
}

/// A value found in memory.
#[derive(Clone, Serialize)]
pub enum Value {
    /// All the values exactly matched this at the time of the scan.
    Exact(Vec<u8>),
    /// The value is not known, so anything represented within this chunk must be considered.
    AnyWithin { memory: Vec<u8>, size: usize },
}
