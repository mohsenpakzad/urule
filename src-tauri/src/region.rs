use crate::scan::Scannable;
use serde::Serialize;
use std::{collections::BTreeMap, mem, ops::Range};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// A memory region.
#[derive(Clone, Serialize)]
pub struct Region<const SIZE: usize, T: Scannable<SIZE>> {
    /// The raw information about this memory region.
    #[serde(skip_serializing)]
    pub info: MEMORY_BASIC_INFORMATION,
    /// Candidate locations that should be considered during subsequent scans.
    pub locations: LocationsStyle<SIZE, T>,
}

unsafe impl<const SIZE: usize, T: Scannable<SIZE>> Send for Region<SIZE, T> {}

impl<const SIZE: usize, T: Scannable<SIZE>> Region<SIZE, T> {
    /// Return the value stored at `addr`.
    pub fn value_at(&self, addr: usize) -> T {
        match &self.locations {
            LocationsStyle::KeyValue(locations) => *locations.get(&addr).unwrap(),
            LocationsStyle::SameValue { value, .. } => *value,
            LocationsStyle::Range { values, .. } => values[addr - self.info.BaseAddress as usize],
            LocationsStyle::Offsetted {
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
            LocationsStyle::Masked {
                values, base, mask, ..
            } => {
                let index = mask
                    .iter()
                    .enumerate()
                    .map(|(index, mask)| (base + index * SIZE, mask))
                    .position(|(address, &mask)| mask && addr == address)
                    .unwrap();
                values[index]
            }
        }
    }
}

/// Locations style for holding our desired locations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum LocationsStyle<const SIZE: usize, T: Scannable<SIZE>> {
    /// A key value locations.
    KeyValue(BTreeMap<usize, T>),
    /// A same value locations.
    SameValue { locations: Vec<usize>, value: T },
    /// A range of memory locations. Everything within here should be considered.
    Range { range: Range<usize>, values: Vec<T> },
    /// A Offsetted memory location. It uses steps to represent addresses.
    Offsetted {
        base: usize,
        offsets: Vec<u16>,
        values: Vec<T>,
    },
    /// A masked memory location. Only items within the mask apply.
    /// The mask assumes 4-byte aligned data  (so one byte for every 4).
    Masked {
        base: usize,
        mask: Vec<bool>,
        values: Vec<T>,
    },
}

impl<const SIZE: usize, T: Scannable<SIZE>> LocationsStyle<SIZE, T> {
    /// Return the amount of locations.
    pub fn len(&self) -> usize {
        match self {
            LocationsStyle::KeyValue(locations) => locations.len(),
            LocationsStyle::SameValue { locations, .. } => locations.len(),
            LocationsStyle::Range { range, .. } => range.len(),
            LocationsStyle::Offsetted { offsets, .. } => offsets.len(),
            LocationsStyle::Masked { values, .. } => values.len(),
        }
    }

    /// Return a iterator over the location addresses.
    pub fn addresses(&self) -> Box<dyn Iterator<Item = usize> + '_> {
        match self {
            LocationsStyle::KeyValue(locations) => Box::new(locations.keys().into_iter().copied()),
            LocationsStyle::SameValue { locations, .. } => Box::new(locations.iter().copied()),
            LocationsStyle::Range { range, .. } => Box::new(range.clone().step_by(SIZE)),
            LocationsStyle::Offsetted { base, offsets, .. } => {
                Box::new(offsets.iter().map(move |&offset| base + offset as usize))
            }
            LocationsStyle::Masked { base, mask, .. } => {
                Box::new(mask.iter().enumerate().filter_map(move |(index, &set)| {
                    if set {
                        Some(base + index * SIZE)
                    } else {
                        None
                    }
                }))
            }
        }
    }

    /// Return the locations based on different location styles.
    pub fn into_locations(self) -> Vec<Location<SIZE, T>> {
        match self {
            LocationsStyle::KeyValue(locations) => locations
                .into_iter()
                .map(|(address, value)| Location { address, value })
                .collect(),
            LocationsStyle::SameValue { locations, value } => locations
                .into_iter()
                .map(|address| Location { address, value })
                .collect(),
            LocationsStyle::Range { range, values } => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| Location {
                    address: range.start + index * SIZE,
                    value,
                })
                .collect(),
            LocationsStyle::Offsetted {
                base,
                offsets,
                values,
            } => offsets
                .into_iter()
                .zip(values)
                .map(|(offset, value)| Location {
                    address: base + offset as usize,
                    value,
                })
                .collect(),
            LocationsStyle::Masked { base, mask, values } => mask
                .into_iter()
                .enumerate()
                .filter_map(|(index, set)| if set { Some(index) } else { None })
                .zip(values)
                .map(|(index, value)| Location {
                    address: base + index * SIZE,
                    value,
                })
                .collect(),
        }
    }

    /// Tries to compact the style into a more efficient representation.
    pub fn try_compact(&mut self) {
        let locations = match self {
            LocationsStyle::KeyValue(locations) if locations.len() > 1 => mem::take(locations),
            _ => return,
        };

        let &low = locations.keys().min().unwrap();
        let &high = locations.keys().max().unwrap();
        let addressing_range = high - low;
        let address_number = addressing_range / SIZE;

        // Would using a byte-mask for the entire region be more worth it?
        // Base(usize) + address_number * mask(bool) < locations.len() * address(usize)
        if mem::size_of::<usize>() + address_number * mem::size_of::<bool>()
            < locations.len() * mem::size_of::<usize>()
        {
            let mut addresses = locations.keys();
            let mut next_set = addresses.next();
            *self = LocationsStyle::Masked {
                base: low,
                mask: (low..high)
                    .step_by(SIZE)
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

        // Can the entire region be represented with a base and 16-bit offsets?
        // And because we ignore locations.len() == 1 cases, if addressing_range is <= u16::MAX
        // Base(usize) + locations.len() * address(u16) < locations.len() * address(usize) is always true
        if addressing_range <= u16::MAX as _ {
            // We will always store a `0` offset, but that's fine, it makes iteration easier and
            // getting rid of it would only gain usu 2 bytes.
            *self = LocationsStyle::Offsetted {
                base: low,
                offsets: locations
                    .keys()
                    .map(|&loc| (loc - low).try_into().unwrap())
                    .collect(),
                values: locations.into_values().collect(),
            };
            return;
        }

        // Neither of the attempts is really better than just storing the locations.
        // Revert to using a discrete representation.
        *self = LocationsStyle::KeyValue(locations);
    }
}

/// Representation of single location in memory.
#[derive(Serialize)]
pub struct Location<const SIZE: usize, T: Scannable<SIZE>> {
    /// Address of the location.
    address: usize,
    /// Value of the location.
    value: T,
}

#[cfg(test)]
mod location_tests {
    use std::collections::BTreeMap;

    use super::*;

    const VALUE: i32 = 3;
    const VALUES: Vec<i32> = Vec::new();

    #[test]
    fn compact_uncompactable() {
        // Same value
        let mut locations = LocationsStyle::SameValue {
            locations: vec![0x2000],
            value: VALUE,
        };
        locations.try_compact();
        assert!(matches!(locations, LocationsStyle::SameValue { .. }));

        // Range
        let mut locations = LocationsStyle::Range {
            range: 0x2000..0x2100,
            values: VALUES,
        };
        locations.try_compact();
        assert!(matches!(locations, LocationsStyle::Range { .. }));

        // Already compacted
        let mut locations = LocationsStyle::Offsetted {
            base: 0x2000,
            offsets: vec![0, 0x20, 0x40],
            values: VALUES,
        };
        locations.try_compact();
        assert!(matches!(locations, LocationsStyle::Offsetted { .. }));

        let mut locations = LocationsStyle::Masked {
            base: 0x2000,
            mask: vec![true, false, false, false],
            values: VALUES,
        };
        locations.try_compact();
        assert!(matches!(locations, LocationsStyle::Masked { .. }));
    }

    #[test]
    fn compact_not_worth() {
        // Too small
        let mut locations = LocationsStyle::KeyValue(BTreeMap::from([(0x2000, 0)]));
        let original = locations.clone();
        locations.try_compact();
        assert_eq!(locations, original);

        // Too sparse and too large to fit in `Offsetted`.
        let mut locations = LocationsStyle::KeyValue(BTreeMap::from([(0x2000, 0), (0x42000, 1)]));
        let original = locations.clone();
        locations.try_compact();
        assert_eq!(locations, original);
    }

    #[test]
    fn compact_offsetted() {
        let mut locations =
            LocationsStyle::KeyValue(BTreeMap::from([(0x2000, 0), (0x2004, 1), (0x2040, 2)]));
        locations.try_compact();
        assert_eq!(
            locations,
            LocationsStyle::Offsetted {
                base: 0x2000,
                offsets: vec![0x0000, 0x0004, 0x0040],
                values: vec![0, 1, 2]
            }
        );
    }

    #[test]
    fn compact_masked() {
        let mut locations = LocationsStyle::KeyValue(BTreeMap::from([
            (0x2000, 0),
            (0x2004, 1),
            (0x200c, 2),
            (0x2010, 3),
            (0x2014, 4),
            (0x2018, 5),
            (0x201c, 6),
            (0x2020, 7),
        ]));
        locations.try_compact();
        assert_eq!(
            locations,
            LocationsStyle::Masked {
                base: 0x2000,
                mask: vec![true, true, false, true, true, true, true, true],
                values: vec![0, 1, 2, 3, 4, 5, 6, 7]
            }
        );
    }

    #[test]
    fn iter_same_value() {
        let locations = LocationsStyle::SameValue {
            locations: vec![0x2000, 0x2004, 0x200c],
            value: VALUE,
        };
        assert_eq!(
            locations.addresses().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c],
        );
    }

    #[test]
    fn iter_key_value() {
        let locations =
            LocationsStyle::KeyValue(BTreeMap::from([(0x2000, 0), (0x2004, 1), (0x200c, 2)]));
        assert_eq!(
            locations.addresses().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c],
        );
    }

    #[test]
    fn iter_offsetted() {
        let locations = LocationsStyle::Offsetted {
            base: 0x2000,
            offsets: vec![0x0000, 0x0004, 0x000c],
            values: VALUES,
        };
        assert_eq!(
            locations.addresses().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }

    #[test]
    fn iter_range() {
        let locations = LocationsStyle::Range {
            range: 0x2000..0x2010,
            values: VALUES,
        };
        assert_eq!(
            locations.addresses().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x2008, 0x200c]
        );
    }

    #[test]
    fn iter_masked() {
        let locations = LocationsStyle::Masked {
            base: 0x2000,
            mask: vec![true, true, false, true],
            values: VALUES,
        };
        assert_eq!(
            locations.addresses().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }
}
