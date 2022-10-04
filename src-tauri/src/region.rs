use serde::Serialize;
use std::mem;
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
    /// Like `Discrete`, but uses less memory.
    SmallDiscrete { base: usize, offsets: Vec<u16> },
    /// A dense memory location. Everything within here should be considered.
    Dense { range: Range<usize>, step: usize },
    /// A sparse memory location. Pretty much like `Dense`, but only items within the mask apply.
    /// The mask assumes 4-byte aligned data  (so one byte for every 4).
    Sparse {
        base: usize,
        mask: Vec<bool>,
        scale: usize,
    },
}

impl CandidateLocations {
    /// Return the amount of candidate locations.
    pub fn len(&self) -> usize {
        match self {
            CandidateLocations::Discrete { locations } => locations.len(),
            CandidateLocations::SmallDiscrete { offsets, .. } => offsets.len(),
            CandidateLocations::Dense { range, step } => range.len() / step,
            CandidateLocations::Sparse { mask, .. } => mask.iter().filter(|x| **x).count(),
        }
    }

    /// Tries to compact the candidate locations into a more efficient representation.
    pub fn try_compact(&mut self, value_size: usize) {
        let locations = match self {
            CandidateLocations::Discrete { locations } if locations.len() >= 2 => {
                mem::take(locations)
            }
            _ => return,
        };

        // It is assumed that locations are always sorted in ascending order.
        let low = *locations.first().unwrap();
        let high = *locations.last().unwrap();
        let size = high - low;
        let size_for_aligned = size / value_size;

        // Can the entire region be represented with a base and 16-bit offsets?
        // And is it more worth than using a single byte per 4-byte aligned location?
        if size <= u16::MAX as _ && locations.len() * mem::size_of::<u16>() < size_for_aligned {
            // We will always store a `0` offset, but that's fine, it makes iteration easier and
            // getting rid of it would only gain usu 2 bytes.
            *self = CandidateLocations::SmallDiscrete {
                base: low,
                offsets: locations
                    .into_iter()
                    .map(|loc| (loc - low).try_into().unwrap())
                    .collect(),
            };
            return;
        }

        // Would using a byte-mask for the entire region be more worth it?
        if size_for_aligned < locations.len() * mem::size_of::<usize>() {
            assert_eq!(low % value_size, 0);

            let mut locations = locations.into_iter();
            let mut next_set = locations.next();
            *self = CandidateLocations::Sparse {
                base: low,
                mask: (low..high)
                    .step_by(value_size)
                    .map(|addr| {
                        if Some(addr) == next_set {
                            next_set = locations.next();
                            true
                        } else {
                            false
                        }
                    })
                    .collect(),
                scale: value_size,
            };
            return;
        }

        // Neither of the attempts is really better than just storing the locations.
        // Revert to using a discrete representation.
        *self = CandidateLocations::Discrete { locations };
    }

    /// Return a iterator over the locations.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        match self {
            CandidateLocations::Discrete { locations } => Box::new(locations.iter().copied()),
            CandidateLocations::SmallDiscrete { base, offsets } => {
                Box::new(offsets.iter().map(move |&offset| base + offset as usize))
            }
            CandidateLocations::Dense { range, step } => Box::new(range.clone().step_by(*step)),
            CandidateLocations::Sparse { base, mask, scale } => Box::new(
                mask.iter()
                    .enumerate()
                    .filter(|(_, &set)| set)
                    .map(move |(i, _)| base + i * scale),
            ),
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