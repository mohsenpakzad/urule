use crate::scan::Scannable;
use log::debug;
use serde::Serialize;
use std::{collections::BTreeMap, mem, ops::Range};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// A memory region.
#[derive(Clone)]
pub struct Region<const SIZE: usize, T: Scannable<SIZE>> {
    /// The raw information about this memory region.
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
            LocationsStyle::Range { range, values } => {
                let index = (addr - range.start) / SIZE;
                values[index]
            }
            LocationsStyle::ExcludedRange {
                range,
                excluded,
                values,
            } => {
                let index = (addr - range.start) / SIZE;
                let smaller_excluded_addresses_count = excluded
                    .iter()
                    .filter(|&&excluded_addr| addr > excluded_addr)
                    .count();
                values[index - smaller_excluded_addresses_count]
            }
            LocationsStyle::Offsetted { base, offsets } => {
                let offset = (addr - base) as u16;
                *offsets.get(&offset).unwrap()
            }
            LocationsStyle::Masked { base, mask, values } => {
                let index = mask
                    .iter()
                    .enumerate()
                    .filter_map(
                        |(index, &set)| {
                            if set {
                                Some(base + index * SIZE)
                            } else {
                                None
                            }
                        },
                    )
                    .position(|address| addr == address)
                    .unwrap();
                values[index]
            }
        }
    }
}

/// Locations style for holding our desired locations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocationsStyle<const SIZE: usize, T: Scannable<SIZE>> {
    /// A key value locations.
    KeyValue(BTreeMap<usize, T>),
    /// A same value locations.
    SameValue { locations: Vec<usize>, value: T },
    /// A range of memory locations. Everything within here should be considered.
    Range { range: Range<usize>, values: Vec<T> },
    /// A excluded range of memory locations. Everything except excluded ones should be considered.
    ExcludedRange {
        range: Range<usize>,
        excluded: Vec<usize>,
        values: Vec<T>,
    },
    /// A Offsetted memory location. It uses steps to represent addresses.
    Offsetted {
        base: usize,
        offsets: BTreeMap<u16, T>,
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
            LocationsStyle::Range { range, .. } => range.len() / SIZE,
            LocationsStyle::ExcludedRange {
                range, excluded, ..
            } => range.len() - excluded.len(),
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
            LocationsStyle::ExcludedRange {
                range, excluded, ..
            } => Box::new(
                range
                    .clone()
                    .step_by(SIZE)
                    .filter(|addr| !excluded.contains(addr)),
            ),
            LocationsStyle::Offsetted { base, offsets, .. } => {
                Box::new(offsets.keys().map(move |&offset| base + offset as usize))
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
            LocationsStyle::ExcludedRange {
                range,
                excluded,
                values,
            } => range
                .step_by(SIZE)
                .filter(|addr| !excluded.contains(addr))
                .zip(values)
                .map(|(address, value)| Location { address, value })
                .collect(),
            LocationsStyle::Offsetted { base, offsets } => offsets
                .into_iter()
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
        let range_max_addresses = (addressing_range / SIZE) + 1;

        // Can the entire region be represented with range style?
        if locations.len() == range_max_addresses {
            debug!("Conversion to LocationsStyle::Range!");
            debug!("Addresses: {}", locations.len());
            debug!("Max addresses: {}", range_max_addresses);
            debug!(
                "Addresses size reduced form {} bytes to {} bytes",
                locations.len() * mem::size_of::<usize>(),
                mem::size_of::<Range<usize>>()
            );

            *self = LocationsStyle::Range {
                range: low..high + 1,
                values: locations.into_values().collect(),
            };
            return;
        }

        // Would using a byte-mask for the entire region be more worth it?
        // Base(usize) + address_number * mask(bool) < locations.len() * address(usize)
        // Due time inefficiency of this method,
        // We only use it on small number of addresses.
        if range_max_addresses <= usize::BITS as _
            && mem::size_of::<usize>() + range_max_addresses
                < locations.len() * mem::size_of::<usize>()
        {
            debug!("Conversion to LocationsStyle::Masked!");
            debug!("Addresses: {}", locations.len());
            debug!("Max addresses: {}", range_max_addresses);
            debug!(
                "Addresses size reduced form {} bytes to {} bytes",
                locations.len() * mem::size_of::<usize>(),
                mem::size_of::<usize>() + range_max_addresses
            );

            let mut addresses = locations.keys();
            let mut next_set = addresses.next();

            *self = LocationsStyle::Masked {
                base: low,
                mask: (low..=high)
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

        // Can the entire region be represented with excluded range style?
        // This method is effective when a small number of range addresses are excluded.
        // Due time inefficiency of this method,
        // We only use it when at least 95% of range_max_addresses is used.
        if locations.len() as f32 >= range_max_addresses as f32 * 0.95 {
            debug!("Conversion to LocationsStyle::ExcludedRange!");
            debug!("Addresses: {}", locations.len());
            debug!("Max addresses: {}", range_max_addresses);

            let excluded = (low..=high)
                .step_by(SIZE)
                .filter(|addr| !locations.contains_key(addr))
                .collect::<Vec<_>>();

            debug!(
                "Addresses size reduced form {} bytes to {} bytes",
                locations.len() * mem::size_of::<usize>(),
                mem::size_of::<Range<usize>>() + excluded.len() * mem::size_of::<usize>()
            );

            *self = LocationsStyle::ExcludedRange {
                range: low..high + 1,
                excluded,
                values: locations.into_values().collect(),
            };
            return;
        }

        // Can the entire region be represented with a base and 16-bit offsets?
        // And because we ignore locations.len() == 1 cases, if addressing_range is <= u16::MAX
        // Base(usize) + locations.len() * address(u16) < locations.len() * address(usize) is always true
        if addressing_range <= u16::MAX as _ {
            debug!("Conversion to LocationsStyle::Offsetted!");
            debug!("Addresses: {}", locations.len());
            debug!("Max addresses: {}", range_max_addresses);
            debug!(
                "Addresses size reduced form {} bytes to {} bytes",
                locations.len() * mem::size_of::<usize>(),
                mem::size_of::<usize>() + locations.len() * mem::size_of::<u16>()
            );

            // We will always store a `0` offset, but that's fine, it makes iteration easier and
            // getting rid of it would only gain usu 2 bytes.
            *self = LocationsStyle::Offsetted {
                base: low,
                offsets: locations
                    .into_iter()
                    .map(|(loc, value)| ((loc - low).try_into().unwrap(), value))
                    .collect(),
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
            offsets: BTreeMap::from([(0, 0), (0x20, 1), (0x40, 2)]),
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
    fn compact_range() {
        let mut locations = LocationsStyle::KeyValue(BTreeMap::from([
            (0x2000, -2),
            (0x2004, -1),
            (0x2008, 0),
            (0x200c, 1),
            (0x2010, 2),
            (0x2014, 3),
            (0x2018, 4),
            (0x201c, 5),
            (0x2020, 6),
        ]));
        locations.try_compact();
        assert_eq!(
            locations,
            LocationsStyle::Range {
                range: 0x2000..0x2021,
                values: vec![-2, -1, 0, 1, 2, 3, 4, 5, 6]
            }
        );
    }

    #[test]
    fn compact_excluded_range() {
        let mut locations = LocationsStyle::KeyValue(
            (0x400..0x481)
                .into_iter()
                .step_by(2)
                .filter_map(|addr| {
                    if addr % 91 == 0 {
                        None
                    } else {
                        Some((addr as usize, addr / 2 as i16))
                    }
                })
                .collect(),
        );
        locations.try_compact();
        assert_eq!(
            locations,
            LocationsStyle::ExcludedRange {
                range: 0x400..0x481,
                excluded: (0x400..=0x480)
                    .into_iter()
                    .step_by(2)
                    .filter(|addr| addr % 91 == 0)
                    .collect::<Vec<_>>(),
                values: (0x400..=0x480)
                    .into_iter()
                    .step_by(2)
                    .filter_map(|addr| {
                        if addr % 91 == 0 {
                            None
                        } else {
                            Some(addr / 2 as i16)
                        }
                    })
                    .collect()
            }
        );
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
                offsets: BTreeMap::from([(0x0000, 0), (0x0004, 1), (0x0040, 2)]),
            }
        );
    }

    #[test]
    fn compact_masked() {
        let mut locations = LocationsStyle::KeyValue(BTreeMap::from([
            (0x2000, 0),
            (0x2004, 1),
            // (0x2008, -1), Not presented
            (0x200c, 2),
            (0x2010, 3),
            (0x2014, 4),
            (0x2018, 5),
            (0x201c, 6),
            // (0x2020, -1), Not presented
            (0x2024, 7),
        ]));
        locations.try_compact();
        assert_eq!(
            locations,
            LocationsStyle::Masked {
                base: 0x2000,
                mask: vec![true, true, false, true, true, true, true, true, false, true],
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
            offsets: BTreeMap::from([(0x0000, 0), (0x0004, 1), (0x000c, 2)]),
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
