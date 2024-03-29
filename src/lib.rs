#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![forbid(clippy::indexing_slicing)]

pub mod bgx;
mod command;
mod con_param;
mod fw;
pub mod mac;
mod response;
mod response_header;
mod scan;
mod scanned_device;
