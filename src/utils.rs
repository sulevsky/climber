use core::ops::Range;

pub fn map_in_range(input: u32, in_range: Range<u32>, out_range: Range<u32>) -> u32 {
    (input - in_range.start) * (out_range.len() as u32) / (in_range.len() as u32) + out_range.start
}
pub const fn parse_bool(input: Option<&str>) -> bool {
    if let Some(inp) = input {
        return matches!(inp.as_bytes(), b"true");
    }
    false
}
pub fn parse_u32(input: Option<&str>, default: u32) -> u32 {
    input
        .map(|s| s.parse::<u32>().unwrap_or(default))
        .unwrap_or(default)
}
