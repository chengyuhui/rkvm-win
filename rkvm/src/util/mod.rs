use std::{ffi::OsStr, os::windows::prelude::*};

pub fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
    string
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
