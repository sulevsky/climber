use core::ops::Range;

pub fn map_in_range(input: u32, in_range: Range<u32>, out_range: Range<u32>) -> u32 {
    (input - in_range.start) * (out_range.len() as u32) / (in_range.len() as u32) + out_range.start
}
