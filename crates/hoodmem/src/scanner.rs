use crate::util::*;
use crate::*;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct KnownIVScanner<T> {
    process: Process,
    regions: Vec<MemoryRegion>,
    pub results: HashMap<u64, (T, Option<T>)>,
    is_new_scan: bool,
}

impl<T> KnownIVScanner<T>
where
    T: Copy + Send + Sync + PartialEq + PartialOrd + std::fmt::Debug,
{
    pub fn new(process: Process) -> Self {
        Self {
            process,
            regions: process.get_writable_regions(),
            results: HashMap::new(),
            is_new_scan: true,
        }
    }

    /// Clears all results and initializes the scanner for the first scan
    pub fn new_scan(&mut self) {
        self.results.clear();
        self.is_new_scan = true;
    }

    /// Narrows down `results` (initally None, which means everything) based on the given value
    pub fn scan(&mut self, value: T) -> Result<()> {
        if self.is_new_scan {
            // Deal with new scans
            for region in self.regions.iter() {
                let region_memory = self
                    .process
                    .read_memory_bytes(region.base_address, region.size as usize);
                if let Ok(region_memory) = region_memory {
                    let scan_range = 0..(region.size as u64 - std::mem::size_of::<T>() as u64);
                    let current_results: Vec<(u64, T)> = scan_range
                        .into_par_iter()
                        .map(|offset| {
                            (
                                region.base_address + offset,
                                read_from_buffer::<T>(&region_memory, offset),
                            )
                        })
                        .filter(|(_, val)| value == *val)
                        .collect();
                    current_results.iter().for_each(|(addr, val)| {
                        self.results.insert(*addr, (*val, None));
                    });
                }
            }
        } else {
            // Filter existing results
            let mut results_so_far: Vec<(u64, T)> = Vec::new();
            for region in self.regions.iter() {
                let region_memory = self
                    .process
                    .read_memory_bytes(region.base_address, region.size as usize)?;
                let scan_range = 0..(region.size as u64 - std::mem::size_of::<T>() as u64);
                let mut current_results: Vec<(u64, T)> = scan_range
                    .into_par_iter()
                    .map(|offset| {
                        (
                            region.base_address + offset,
                            read_from_buffer::<T>(&region_memory, offset),
                        )
                    })
                    .filter(|(_, val)| value == *val)
                    .collect();

                results_so_far.append(&mut current_results);
            }
            self.results
                .keys()
                .into_iter()
                .map(|x| *x)
                .collect::<Vec<u64>>()
                .iter()
                .for_each(|k| {
                    if let Some((_, val)) = (&results_so_far)
                        .into_par_iter()
                        .find_any(|(addr, _)| *addr == *k)
                    {
                        // Update existing keys which were found in the new results
                        let prev = (*self.results.get(k).unwrap()).0;
                        self.results.insert(k.clone(), (*val, Some(prev)));
                    } else {
                        // Remove keys which weren't found in the new results
                        self.results.remove(k);
                    }
                });
        }

        self.is_new_scan = false;
        Ok(())
    }
}
