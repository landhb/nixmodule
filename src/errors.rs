use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use std::error::Error;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum NixModuleError {
    BadFilePath,
    QemuError,
    SshError,
    BuildError,
    InsmodError,
    TestError,
    TimeoutError,
}

impl Display for NixModuleError {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> Result {
        <NixModuleError as Debug>::fmt(self, f)
    }
}

impl Error for NixModuleError {}
