use serde::Serialize;
use std::{collections::BTreeMap, mem, ops::Range};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// A memory region.
#[derive(Clone, Serialize)]
pub struct Region {
    /// The raw information about this memory region.
    #[serde(skip_serializing)]
    pub info: MEMORY_BASIC_INFORMATION,
    /// Candidate locations that should be considered during subsequent scans.
    pub locations: CandidateLocations,
}
unsafe impl Send for Region {}

impl Region {
    /// Return the value stored at `addr`.
    pub fn value_at(&self, addr: usize) -> i32 {
        match &self.locations {
            CandidateLocations::KeyValue(locations) => *locations.get(&addr).unwrap(),
            CandidateLocations::SameValue { value, .. } => *value,
            CandidateLocations::Range { values, .. } => {
                values[addr - self.info.BaseAddress as usize]
            }
            CandidateLocations::Offsetted {
                values,
                base,
                offsets,
            } => {
                let index = offsets
                    .iter()
                    .position(|&offset| base + offset as usize == addr)
                    .unwrap();
                values[index]
            }
            CandidateLocations::Masked { values, base, mask } => {
                let index = mask
                    .iter()
                    .enumerate()
                    .map(|(index, mask)| (base + index * 4, mask))
                    .position(|(address, &mask)| mask && addr == address)
                    .unwrap();
                values[index]
            }
        }
    }
}

/// Candidate memory locations for holding our desired value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum CandidateLocations {
    /// A key value locations.
    KeyValue(BTreeMap<usize, i32>),
    /// A same value locations.
    SameValue { locations: Vec<usize>, value: i32 },
    /// A range of memory locations. Everything within here should be considered.
    Range {
        range: Range<usize>,
        values: Vec<i32>,
    },
    /// A Offsetted memory location. It uses steps to represent addresses.
    // TODO this could also assume 4-byte aligned so we'd gain 2 bits for offsets.
    Offsetted {
        base: usize,
        offsets: Vec<u16>,
        values: Vec<i32>,
    },
    /// A masked memory location. Only items within the mask apply.
    /// The mask assumes 4-byte aligned data  (so one byte for every 4).
    Masked {
        base: usize,
        mask: Vec<bool>,
        values: Vec<i32>,
    },
}

impl CandidateLocations {
    /// Return the amount of candidate locations.
    pub fn len(&self) -> usize {
        match self {
            CandidateLocations::KeyValue(locations) => locations.len(),
            CandidateLocations::SameValue { locations, .. } => locations.len(),
            CandidateLocations::Range { range, .. } => range.len(),
            CandidateLocations::Offsetted { offsets, .. } => offsets.len(),
            CandidateLocations::Masked { values, .. } => values.len(),
        }
    }

    /// Tries to compact the candidate locations into a more efficient representation.
    pub fn try_compact(&mut self) {
        let locations = match self {
            CandidateLocations::KeyValue(locations) if locations.len() > 1 => mem::take(locations),
            _ => return,
        };

        // It is assumed that locations are always sorted in ascending order.
        let &low = locations.keys().min().unwrap();
        let &high = locations.keys().max().unwrap();
        let size = high - low;

        // Can the entire region be represented with a base and 16-bit offsets?
        // And is it more worth than using a single byte per 4-byte aligned location?
        if size <= u16::MAX as _ && locations.len() * mem::size_of::<u16>() < size / 4 {
            // We will always store a `0` offset, but that's fine, it makes iteration easier and
            // getting rid of it would only gain usu 2 bytes.
            *self = CandidateLocations::Offsetted {
                base: low,
                offsets: locations
                    .keys()
                    .map(|&loc| (loc - low).try_into().unwrap())
                    .collect(),
                values: locations.into_values().collect(),
            };
            return;
        }

        // // Would using a byte-mask for the entire region be more worth it?
        if size / 4 < locations.len() * mem::size_of::<usize>() {
            assert_eq!(low % 4, 0);

            let mut addresses = locations.keys();
            let mut next_set = addresses.next();
            *self = CandidateLocations::Masked {
                base: low,
                mask: (low..high)
                    .step_by(4)
                    .map(|addr| {
                        if Some(&addr) == next_set {
                            next_set = addresses.next();
                            true
                        } else {
                            false
                        }
                    })
                    .collect(),
                values: locations.into_values().collect(),
            };
            return;
        }

        // Neither of the attempts is really better than just storing the locations.
        // Revert to using a discrete representation.
        *self = CandidateLocations::KeyValue(locations);
    }

    /// Return a iterator over the locations.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        match self {
            CandidateLocations::KeyValue(locations) => {
                Box::new(locations.keys().into_iter().copied())
            }
            CandidateLocations::SameValue { locations, .. } => Box::new(locations.iter().copied()),
            CandidateLocations::Range { range, .. } => Box::new(range.clone().step_by(4)),
            CandidateLocations::Offsetted { base, offsets, .. } => {
                Box::new(offsets.iter().map(move |&offset| base + offset as usize))
            }
            CandidateLocations::Masked { base, mask, .. } => Box::new(
                mask.iter()
                    .enumerate()
                    .filter(|(_, &set)| set)
                    .map(move |(i, _)| base + i * 4),
            ),
        }
    }
}
