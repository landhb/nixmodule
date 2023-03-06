use crate::errors::NixModuleError::*;
use crate::utils::print_output;
use crate::KConfig;
use std::error::Error;
use std::process::Command;

pub struct ModuleBuilder;

impl ModuleBuilder {
    pub fn build(
        name: &str,
        defines: &Option<Vec<String>>,
        kernel: &KConfig,
    ) -> Result<String, Box<dyn Error>> {
        let builddir = std::env::var("PWD")?;
        let mut make = Command::new("make");

        if let Some(definition) = defines {
            let mut tuples: Vec<(&str, &str)> = Vec::new();
            for v in definition.iter() {
                let mut inner = v.split('=');
                let name = inner.next().ok_or(BuildError)?;
                let value = inner.next().ok_or(BuildError)?;
                tuples.push((name, value));
            }
            make.envs(tuples);
        }

        let res = make
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
