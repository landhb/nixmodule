use colored::*;
use home_dir::HomeDirExt;
use prettytable::{Row, Table};
use serde::Deserialize;
use std::error::Error;
use std::fs::read;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[macro_use]
mod macros;

mod cache;
use cache::Cache;

mod errors;
use errors::NixModuleError::{self, *};

mod qemu;
use qemu::Qemu;

mod builder;
use builder::ModuleBuilder;

#[macro_use]
extern crate prettytable;

#[derive(StructOpt, Debug)]
#[structopt(name = "nixmodule")]
struct Opt {
    #[structopt(
        short = "c",
        long = "config",
        default_value = "./nixmodule-config.toml"
    )]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    cache: PathBuf,
    module: Module,
    kernels: Vec<KConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct DiskImage {
    name: String,
    url_base: String,
    path: String,
    sshkey: String,
}

#[derive(Debug, Deserialize)]
pub struct KConfig {
    version: String,
    url_base: String,
    headers: String,
    kernel: String,
    disk: DiskImage,
    runner: String,
    runner_extra_args: Option<Vec<String>>,
}

fn test(name: &str, kernel: &KConfig, handle: &Qemu) -> Result<(), Box<dyn Error>> {
    // Compile the module against the headers
    let build = ModuleBuilder::build(name, &kernel)?;
    log_success!("Build success for kernel {:?}", kernel.version);

    // Upload the module
    let uploaded = format!(
        "/tmp/{:?}",
        Path::new(&build).file_name().ok_or(BadFilePath)?
    );
    handle.transfer(&build, &uploaded)?;
    log_status!("Uploaded {}", uploaded);
    handle
        .runcmd(&format!("insmod {} ports=8000", uploaded))
        .or(Err(InsmodError))?;
    log_success!("Insmod successful!");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Obtain the running config
    let opt = Opt::from_args();
    let config: Config = toml::from_slice(&read(opt.config)?)?;

    println!("{:#?}", config);

    // Init the cache
    let cache = Cache::new(&config.cache.expand_home()?);

    // Results table
    let mut table = Table::new();
    table.add_row(row![Fy->"Version", Fy->"Build", Fy->"Insmod", Fy->"Tests"]);

    for mut kernel in config.kernels {
        // Download or retrieve cached items
        cache.get(&mut kernel)?;

        // Start qemu with the config
        let handle = Qemu::start(&kernel)?;

        let mut row = row![kernel.version, "N/A".blue(), "N/A".blue(), "N/A".blue()];

        match test(&config.module.name, &kernel, &handle) {
            Err(x) if x.downcast_ref::<NixModuleError>() == Some(&BuildError) => {
                row.set_cell(cell!("Failed".red()), 1);
            }
            Err(x) if x.downcast_ref::<NixModuleError>() == Some(&InsmodError) => {
                row.set_cell(cell!("Ok".green()), 1);
                row.set_cell(cell!("Failed".red()), 2);
            }
            Err(x) if x.downcast_ref::<NixModuleError>() == Some(&TestError) => {
                row.set_cell(cell!("Ok".green()), 1);
                row.set_cell(cell!("Ok".green()), 2);
                row.set_cell(cell!("Failed".red()), 3);
            }
            Ok(_) => {
                row.set_cell(cell!("Ok".green()), 1);
                row.set_cell(cell!("Ok".green()), 2);
                row.set_cell(cell!("Ok".green()), 3);
            }
            _ => {}
        }
        table.add_row(row);

        // Wait and stop qemu
        std::thread::sleep(std::time::Duration::new(3, 0));
        handle.stop()?;
    }

    table.printstd();
    Ok(())
}
