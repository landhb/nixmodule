use colored::*;
use home_dir::HomeDirExt;
use prettytable::Table;
use serde::Deserialize;
use std::error::Error;
use std::fs::read;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[macro_use]
mod utils;

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
    /// Path to a configuration file
    #[structopt(
        short = "c",
        long = "config",
        default_value = "./nixmodule-config.toml"
    )]
    config: PathBuf,

    /// Run suite for a specific kernel version
    /// or any kernel that starts with this value
    /// (i.e 5 will run every 5.X.X vs 5.1 which will
    /// only run 5.1*)
    #[structopt(short = "k", long = "kernel")]
    kernel: Option<String>,

    /// Enter a shell on the box, also starts qemu with
    /// gdb. Performs the build + setup stages first.
    #[structopt(short = "d", long = "debug")]
    debug: bool,
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
    test_script: UploadFile,
    insmod_args: String,
    test_files: Vec<UploadFile>,
}

#[derive(Debug, Deserialize)]
pub struct UploadFile {
    local: String,
    remote: String,
}

#[derive(Debug, Deserialize)]
pub struct DiskImage {
    url_base: String,
    path: String,
    sshkey: String,
    boot: String,
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

    // Allow users to disable kvm
    #[serde(default = "enable_kvm")]
    kvm: bool,

    // Allow users to increase timeout
    timeout: Option<u64>,
}

/// Speed things up by enabling by default
fn enable_kvm() -> bool {
    true
}

/// Run through the test
fn test(
    module: &Module,
    kernel: &KConfig,
    handle: &Qemu,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    log_status!("Building module for {}", kernel.version);

    // Compile the module against the headers
    let build = ModuleBuilder::build(&module.name, &kernel)?;
    log_success!("Build success for kernel {:?}", kernel.version);

    // Upload the module
    let uploaded = format!(
        "/tmp/{:?}",
        Path::new(&build).file_name().ok_or(BadFilePath)?
    );
    handle.transfer(&build, &uploaded).or(Err(InsmodError))?;
    log_status!("Uploaded {}", uploaded);

    // Perform insmod
    if !debug {
        handle
            .runcmd(&format!("insmod {} {}", uploaded, module.insmod_args))
            .or(Err(InsmodError))?;
        log_success!("Insmod successful for {}!", kernel.version);
    }

    // Upload all test files
    handle
        .transfer(&module.test_script.local, &module.test_script.remote)
        .or(Err(TestError))?;
    for upload in &module.test_files {
        handle.transfer(&upload.local, &upload.remote)?;
    }

    // Run the test script or enter an interactive session
    if !debug {
        handle
            .runcmd(&module.test_script.remote)
            .or(Err(TestError))?;
        log_success!("Test successful for {}!", kernel.version);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Return an appropriate exit code
    let mut exitcode = Success as _;

    // Obtain the running config
    let opt = Opt::from_args();
    let mut config: Config = toml::from_slice(&read(opt.config)?)?;

    // Init the cache
    let cache = Cache::new(&config.cache.expand_home()?);

    // Results table
    let mut table = Table::new();
    table.add_row(row![Fy->"Version", Fy->"Build", Fy->"Insmod", Fy->"Tests"]);

    // Optionally filter for specific version
    let kernel_iter: Box<dyn Iterator<Item = &mut KConfig>> = match opt.kernel {
        Some(_) => Box::new(
            config
                .kernels
                .iter_mut()
                .filter(|v| v.version.starts_with(opt.kernel.as_ref().unwrap())),
        ),
        None => Box::new(config.kernels.iter_mut()),
    };

    for mut kernel in kernel_iter {
        // Download or retrieve cached items
        cache.get(&mut kernel)?;

        // Start qemu with the config
        let handle = Qemu::start(&kernel, opt.debug)?;

        // Create results row
        let mut row = row![kernel.version, Fb->"N/A", "N/A".blue(), "N/A".blue()];
        match test(&config.module, &kernel, &handle, opt.debug) {
            Err(x) if x.downcast_ref::<NixModuleError>() == Some(&BuildError) => {
                row.set_cell(cell!(Fr->"Failed"), 1)?;
                exitcode = BuildError as _;
            }
            Err(x) if x.downcast_ref::<NixModuleError>() == Some(&InsmodError) => {
                row.set_cell(cell!(Fg->"Ok"), 1)?;
                row.set_cell(cell!(Fr->"Failed"), 2)?;
                exitcode = InsmodError as _;
            }
            Err(x) if x.downcast_ref::<NixModuleError>() == Some(&TestError) => {
                row.set_cell(cell!(Fg->"Ok"), 1)?;
                row.set_cell(cell!(Fg->"Ok"), 2)?;
                row.set_cell(cell!(Fr->"Failed"), 3)?;
                exitcode = TestError as _;
            }
            Ok(_) => {
                row.set_cell(cell!(Fg->"Ok"), 1)?;
                row.set_cell(cell!(Fg->"Ok"), 2)?;
                row.set_cell(cell!(Fg->"Ok"), 3)?;
            }
            _ => {}
        }
        table.add_row(row);

        // Go interactive if a debug session was requested
        if opt.debug {
            handle.interact().unwrap_or_else(|e| println!("{:?}", e));
        }

        // Wait and stop qemu
        handle.stop()?;
    }

    if !opt.debug {
        table.printstd();
    }

    std::process::exit(exitcode);
}
