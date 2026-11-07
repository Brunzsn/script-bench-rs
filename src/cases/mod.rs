mod sort_userdata;

use crate::harness::Case;

/// Every benchmark case. Add one entry here per case.
pub static CASES: &[Case] = &[sort_userdata::CASE];
