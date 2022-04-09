use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use std::error::Error;

#[allow(dead_code)]
#[derive(Debug)]
pub enum NixModuleError {
    BadFilePath,
}

impl Display for NixModuleError {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> Result {
        <NixModuleError as Debug>::fmt(self, f)
    }
}

impl Error for NixModuleError {}
