use crate::errors::NixModuleError::*;
use crate::KConfig;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct ModuleBuilder;

impl ModuleBuilder {
    pub fn build(name: &str, kernel: &KConfig) -> Result<String, Box<dyn Error>> {
        let builddir = std::env::var("PWD")?;
        let res = Command::new("make")
            .current_dir(&builddir)
            .arg(format!("KERNEL={}", kernel.headers))
            .arg(format!("TARGET={}-{}", name, kernel.version))
            .output()?;

        match res.status.success() {
            true => Ok(format!("{}/{}-{}.ko", builddir, name, kernel.version)),
            false => {
                println!("{:?}", std::str::from_utf8(&res.stderr)?);
                Err(BuildError.into())
            }
        }
    }
}
