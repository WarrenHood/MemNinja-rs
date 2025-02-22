use crate::util::*;
use crate::*;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

/// Scan filter used when diffing memory and updating scan results
#[derive(Clone, Copy, Debug)]
pub enum ScanFilter<T> {
    Exact(T),
    /// Approximately equals, within a threshold
    Approximate(T, T),
    Increased,
    Decreased,
    IncreasedBy(T),
    DecreasedBy(T),
    IncreasedByAtLeast(T),
    IncreasedByAtMost(T),
    DecreasedByAtLeast(T),
    DecreasedByAtMost(T),
    Changed,
    Unchanged,
    ChangedByAtLeast(T),
    ChangedByAtMost(T),
    UnchangedByAtLeast(T),
    UnchangedByAtMost(T),
    Unknown,
}

impl<T> ScanFilter<T>
where
    T: Copy + PartialOrd + PartialEq + std::ops::Sub<Output = T> + std::ops::Add<Output = T>,
{
    pub fn matches(&self, new_t: &T, old_t: &T) -> bool {
        match self {
            ScanFilter::Exact(new_value) => *new_value == *new_t,
            ScanFilter::Approximate(new_value, threshold) => {
                (if *new_t > *new_value {
                    *new_t - *new_value
                } else {
                    *new_value - *new_t
                }) <= *threshold
            }
            ScanFilter::Increased => *new_t > *old_t,
            ScanFilter::Decreased => *new_t < *old_t,
            ScanFilter::IncreasedBy(diff) => *new_t == (*old_t + *diff),
            ScanFilter::DecreasedBy(diff) => *new_t == *old_t - *diff,
            ScanFilter::IncreasedByAtLeast(diff) => *new_t >= *old_t && *new_t - *old_t >= *diff,
            ScanFilter::IncreasedByAtMost(diff) => *new_t >= *old_t && *new_t - *old_t <= *diff,
            ScanFilter::DecreasedByAtLeast(diff) => *new_t <= *old_t && *old_t - *new_t >= *diff,
            ScanFilter::DecreasedByAtMost(diff) => *new_t <= *old_t && *old_t - *new_t <= *diff,
            ScanFilter::Changed => *new_t != *old_t,
            ScanFilter::Unchanged => *new_t == *old_t,
            ScanFilter::ChangedByAtLeast(diff) => {
                (if *new_t > *old_t {
                    *new_t - *old_t
                } else {
                    *old_t - *new_t
                }) >= *diff
            }
            ScanFilter::ChangedByAtMost(diff) => {
                (if *new_t > *old_t {
                    *new_t - *old_t
                } else {
                    *old_t - *new_t
                }) <= *diff
            }
            ScanFilter::UnchangedByAtLeast(diff) => {
                (if *new_t > *old_t {
                    *new_t - *old_t
                } else {
                    *old_t - *new_t
                }) <= *diff
            }
            ScanFilter::UnchangedByAtMost(diff) => {
                (if *new_t > *old_t {
                    *new_t - *old_t
                } else {
                    *old_t - *new_t
                }) <= *diff
            }
            ScanFilter::Unknown => true,
        }
    }
}

/// Region scan results
///
/// Will store entire regions of memory
pub struct RegionResults {
    /// Region base address
    region: MemoryRegion,
    /// Offsets of current hits within this region
    hit_offsets: Option<Vec<usize>>,
    /// The last snapshot of this memory region (prev values)
    buffer: Option<Vec<u8>>,
}

impl RegionResults {
    /// Creates a new RegionResults object
    pub fn new(region: MemoryRegion) -> Self {
        Self {
            region,
            hit_offsets: None,
            buffer: None,
        }
    }

    pub fn get_results<T: Copy + Send + Sync>(&self) -> Option<Vec<(usize, T)>> {
        let size_of_t = std::mem::size_of::<T>();
        if let Some(offsets) = self.hit_offsets.as_ref() {
            if let Some(buffer) = self.buffer.as_ref() {
                return Some(
                    offsets
                        .into_par_iter()
                        // Ensure we don't read any results outside the buffer
                        .filter(|offset| **offset + size_of_t <= buffer.len())
                        .map(|offset| {
                            (
                                *offset + self.region.base_address,
                                read_from_buffer::<T>(buffer, *offset),
                            )
                        })
                        .collect(),
                );
            }
        }
        None
    }

    pub fn print<T: std::fmt::Debug + Copy>(&self) {
        let results_count = if self.hit_offsets.is_some() {
            self.hit_offsets.as_ref().unwrap().len()
        } else {
            0
        };
        println!(
            "[Region 0x{:016x} - 0x{:016x}] {} Results:",
            self.region.base_address,
            self.region.base_address + self.region.size,
            results_count
        );
        if let Some(offsets) = &self.hit_offsets {
            for offset in offsets.iter().take(1) {
                if let Some(buffer) = &self.buffer {
                    println!(
                        "0x{:016x} = {:#?}",
                        *offset + self.region.base_address,
                        read_from_buffer::<T>(buffer, *offset)
                    );
                }
            }
        }
        println!("-----------------------------------------------------------------------------")
    }

    /// Clear these results for the next scan
    pub fn clear(&mut self) {
        self.hit_offsets = None;
        self.buffer = None;
    }

    /// Updates results given a buffer of this regions new memory, and a filter
    pub fn update_results<T>(&mut self, region_buf: Vec<u8>, filter: ScanFilter<T>)
    where
        T: Copy
            + Send
            + Sync
            + PartialOrd
            + PartialEq
            + std::ops::Sub<Output = T>
            + std::ops::Add<Output = T>,
    {
        let size_of_t = std::mem::size_of::<T>();
        if self.buffer.is_none() {
            // There was no previous buffer, this must be the first scan
            match filter {
                // At least filter on exact value first scans (known initial value)
                ScanFilter::Exact(_) => {
                    // New exact value scan
                    let scan_range = 0..(self.region.size - std::mem::size_of::<T>());
                    self.hit_offsets = Some(
                        scan_range
                            .into_par_iter()
                            // Ensure we don't read any results outside the buffer
                            .filter(|offset| *offset + size_of_t <= region_buf.len())
                            .map(|offset| (offset, read_from_buffer::<T>(&region_buf, offset)))
                            .filter(|(_, val)| filter.matches(val, val))
                            .map(|(addr, _)| addr)
                            .collect(),
                    );
                }
                _ => {}
            }
        } else {
            // Subsequent scans. We have access to previous values here
            let scan_range = 0..(self.region.size - std::mem::size_of::<T>());

            if self.hit_offsets.is_some() {
                // We have existing hits, filter on them
                self.hit_offsets = Some(
                    self.hit_offsets
                        .as_ref()
                        .unwrap()
                        .into_par_iter()
                        // Ensure we don't read any results outside the buffer
                        .filter(|offset| **offset + size_of_t <= region_buf.len())
                        .map(|offset| {
                            (
                                offset,
                                read_from_buffer::<T>(&region_buf, *offset),
                                read_from_buffer(self.buffer.as_ref().unwrap(), *offset),
                            )
                        })
                        .filter(|(_, val, prev)| filter.matches(val, prev))
                        .map(|(addr, _, _)| *addr)
                        .collect(),
                );
            } else {
                // No existing hits, accept any that match the filter within the scan range
                self.hit_offsets = Some(
                    scan_range
                        .into_par_iter()
                        // Ensure we don't read any results outside the buffer
                        .filter(|offset| *offset + size_of_t <= region_buf.len())
                        .map(|offset| {
                            (
                                offset,
                                read_from_buffer::<T>(&region_buf, offset),
                                read_from_buffer::<T>(self.buffer.as_ref().unwrap(), offset),
                            )
                        })
                        .filter(|(_, val, prev)| filter.matches(val, prev))
                        .map(|(addr, _, _)| addr)
                        .collect(),
                )
            }
        }
        if self.hit_offsets.as_ref().is_none() || self.hit_offsets.as_ref().unwrap().len() > 0 {
            // Only keep track of previous values if we have hits, or haven't scanned yet
            self.buffer = Some(region_buf)
        } else {
            // Hit offsets length is 0
            self.buffer = None
        }
    }
}

pub struct Scanner {
    process: Arc<dyn Process>,
    pub results: HashMap<MemoryRegion, RegionResults>,
    is_new_scan: bool,
}

impl Scanner {
    pub fn new(process: Arc<dyn Process>) -> Self {
        Self {
            process,
            results: HashMap::new(),
            is_new_scan: true,
        }
    }

    /// Count the number of results so far
    pub fn count_results(&self) -> Option<usize> {
        if self.is_new_scan {
            return None;
        }
        let hit_offsets: Vec<&Vec<usize>> = self
            .results
            .values()
            .into_iter()
            .map(|result| result.hit_offsets.as_ref())
            .filter(|hit_offsets| hit_offsets.is_some())
            .map(|hit_offsets| hit_offsets.unwrap())
            .collect();

        if hit_offsets.len() == 0 {
            // We have not yet scanned anything
            return None;
        }

        Some(
            hit_offsets
                .iter()
                .map(|hit_offsets| hit_offsets.len())
                .sum(),
        )
    }

    /// Gets all scan results
    pub fn get_results<T>(&self) -> Vec<(usize, T)>
    where
        T: Copy + Send + Sync,
    {
        self.results
            .values()
            .into_iter()
            .map(|results| results.get_results::<T>())
            .filter(|results| results.is_some())
            .flat_map(|results| results.unwrap())
            .collect()
    }

    /// Gets first `n` scan results
    pub fn get_first_results<T>(&self, n: usize) -> Vec<(usize, T)>
    where
        T: Copy + Send + Sync,
    {
        self.results
            .values()
            .into_iter()
            .map(|results| results.get_results::<T>())
            .filter(|results| results.is_some())
            .flat_map(|results| results.unwrap())
            .take(n)
            .collect()
    }

    /// Gets the result at a given index
    /// Warning: This is bad for performance, prefer getting a range of results
    pub fn get_nth_result<T>(&self, n: usize) -> Option<(usize, T)>
    where
        T: Copy + Send + Sync,
    {
        self.results
            .values()
            .into_iter()
            .map(|results| results.get_results::<T>())
            .filter(|results| results.is_some())
            .flat_map(|results| results.unwrap())
            .nth(n)
    }

    /// Gets a range of results
    pub fn get_results_range<T>(&self, start_index: usize, end_index: usize) -> Vec<(usize, T)>
    where
        T: Copy + Send + Sync,
    {
        self.results
            .values()
            .into_iter()
            .map(|results| results.get_results::<T>())
            .filter(|results| results.is_some())
            .flat_map(|results| results.unwrap())
            .skip(start_index)
            .take(end_index - start_index + 1)
            .collect()
    }

    /// Clears all results and initializes the scanner for the first scan
    pub fn new_scan(&mut self) {
        self.results.clear();
        self.is_new_scan = true;
    }

    /// Narrows down `results` (initally None, which means everything) based on the given value
    pub fn scan<T>(&mut self, filter: ScanFilter<T>) -> Result<()>
    where
        T: Copy
            + std::fmt::Debug
            + Send
            + Sync
            + PartialOrd
            + PartialEq
            + std::ops::Sub<Output = T>
            + std::ops::Add<Output = T>,
    {
        // println!("Performing scan with filter: {:?}", filter);
        let regions = self.process.get_writable_regions();

        // println!("Writable regions:");
        // for region in regions.iter() {
        //     println!(
        //         " {:016x} - {:016x}",
        //         region.base_address,
        //         region.base_address + region.size - 1
        //     )
        // }

        if self.is_new_scan {
            // Deal with new scans
            for region in regions.iter() {
                let region_memory = self
                    .process
                    .read_memory_bytes(region.base_address, region.size);
                if let Ok(region_memory) = region_memory {
                    self.results.insert(*region, RegionResults::new(*region));
                    self.results
                        .get_mut(region)
                        .unwrap()
                        .update_results(region_memory, filter);
                }
            }
        } else {
            // Filter existing results
            for region in &regions {
                if let Some(region_results) = self.results.get_mut(&region) {
                    if region_results.hit_offsets.as_ref().is_none()
                        || region_results.hit_offsets.as_ref().unwrap().len() > 0
                    {
                        // Only bother to update memory of things with no hit results yet, or with hit results of length > 0
                        let region_memory = self
                            .process
                            .read_memory_bytes(region.base_address, region.size as usize);
                        if let Ok(region_memory) = region_memory {
                            region_results.update_results(region_memory, filter);
                        }
                    }
                }
            }
        }

        self.is_new_scan = false;
        Ok(())
    }
}
