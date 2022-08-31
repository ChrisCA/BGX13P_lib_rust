#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![forbid(clippy::indexing_slicing)]

pub mod bgx;
mod command;
mod fw;
mod response;
mod response_header;
mod scan;
mod scanned_device;
