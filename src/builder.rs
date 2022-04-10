use crate::errors::NixModuleError::*;
use crate::utils::print_output;
use crate::KConfig;
use std::error::Error;
use std::process::Command;

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
                print_output(std::str::from_utf8(&res.stdout)?);
                print_output(std::str::from_utf8(&res.stderr)?);
                Err(BuildError.into())
            }
        }
    }
}
