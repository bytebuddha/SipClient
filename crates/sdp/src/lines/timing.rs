use nom::character::is_digit;
use nirah_parse::parse_u64;

use crate::SdpSessionAttributes;

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct SdpTiming(u64, u64);

named!(pub parse_timing<SdpTiming>, do_parse!(
    start: map_res!(take_while!(is_digit), parse_u64) >>
    char!(' ') >>
    end: map_res!(take_while!(is_digit), parse_u64) >>
    (SdpTiming(start, end))
));

impl fmt::Display for SdpTiming {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

named!(pub parse_time_line<SdpSessionAttributes>, do_parse!(
    tag!("t=") >>
    timing: parse_timing >>
    tag!("\r\n") >>
    (SdpSessionAttributes::Timing(timing))
));

impl SdpTiming {

    pub fn new(start: u64, end: u64) -> SdpTiming {
        SdpTiming(start, end)
    }
}