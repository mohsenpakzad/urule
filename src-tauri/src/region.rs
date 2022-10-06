use serde::Serialize;
use std::{mem, ops::Range};
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
    pub fn value_at(&self, addr: usize) -> i32 {
        match &self.value {
            Value::Exact(n) => *n,
            Value::AnyWithin(chunk) => {
                let base = addr - self.info.BaseAddress as usize;
                let bytes = &chunk[base..base + 4];
                i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
            }
        }
    }
}

/// Candidate memory locations for holding our desired value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum CandidateLocations {
    /// A same value locations.
    ///
    /// It is a logic error to have the locations in non-ascending order.
    SameValue { locations: Vec<usize> },
    /// A Offsetted memory location. It uses steps to represent addresses.
    // TODO this could also assume 4-byte aligned so we'd gain 2 bits for offsets.
    Offsetted { base: usize, offsets: Vec<u16> },
    /// A masked memory location. Only items within the mask apply.
    /// The mask assumes 4-byte aligned data  (so one byte for every 4).
    Masked { base: usize, mask: Vec<bool> },
    /// A range of memory locations. Everything within here should be considered.
    Range { range: Range<usize> },
}

impl CandidateLocations {
    /// Return the amount of candidate locations.
    pub fn len(&self) -> usize {
        match self {
            CandidateLocations::SameValue { locations } => locations.len(),
            CandidateLocations::Offsetted { offsets, .. } => offsets.len(),
            CandidateLocations::Range { range } => range.len(),
            CandidateLocations::Masked { mask, .. } => mask.iter().filter(|x| **x).count(),
        }
    }

    /// Tries to compact the candidate locations into a more efficient representation.
    pub fn try_compact(&mut self) {
        let locations = match self {
            CandidateLocations::SameValue { locations } if locations.len() >= 2 => {
                mem::take(locations)
            }
            _ => return,
        };

        // It is assumed that locations are always sorted in ascending order.
        let low = *locations.first().unwrap();
        let high = *locations.last().unwrap();
        let size = high - low;

        // Can the entire region be represented with a base and 16-bit offsets?
        // And is it more worth than using a single byte per 4-byte aligned location?
        if size <= u16::MAX as _ && locations.len() * mem::size_of::<u16>() < size / 4 {
            // We will always store a `0` offset, but that's fine, it makes iteration easier and
            // getting rid of it would only gain usu 2 bytes.
            *self = CandidateLocations::Offsetted {
                base: low,
                offsets: locations
                    .into_iter()
                    .map(|loc| (loc - low).try_into().unwrap())
                    .collect(),
            };
            return;
        }

        // Would using a byte-mask for the entire region be more worth it?
        if size / 4 < locations.len() * mem::size_of::<usize>() {
            assert_eq!(low % 4, 0);

            let mut locations = locations.into_iter();
            let mut next_set = locations.next();
            *self = CandidateLocations::Masked {
                base: low,
                mask: (low..high)
                    .step_by(4)
                    .map(|addr| {
                        if Some(addr) == next_set {
                            next_set = locations.next();
                            true
                        } else {
                            false
                        }
                    })
                    .collect(),
            };
            return;
        }

        // Neither of the attempts is really better than just storing the locations.
        // Revert to using a discrete representation.
        *self = CandidateLocations::SameValue { locations };
    }

    /// Return a iterator over the locations.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        match self {
            CandidateLocations::SameValue { locations } => Box::new(locations.iter().copied()),
            CandidateLocations::Offsetted { base, offsets } => {
                Box::new(offsets.iter().map(move |&offset| base + offset as usize))
            }
            CandidateLocations::Range { range } => Box::new(range.clone().step_by(4)),
            CandidateLocations::Masked { base, mask } => Box::new(
                mask.iter()
                    .enumerate()
                    .filter(|(_, &set)| set)
                    .map(move |(i, _)| base + i * 4),
            ),
        }
    }
}

/// A value found in memory.
#[derive(Clone, Serialize)]
pub enum Value {
    /// All the values exactly matched this at the time of the scan.
    Exact(i32),
    /// The value is not known, so anything represented within this chunk must be considered.
    AnyWithin(Vec<u8>),
}
