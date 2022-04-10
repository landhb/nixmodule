use home_dir::HomeDirExt;
use serde::Deserialize;
use std::error::Error;
use std::fs::read;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod cache;
use cache::Cache;

mod errors;
use errors::NixModuleError::*;

mod qemu;
use qemu::Qemu;

mod builder;
use builder::ModuleBuilder;

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

fn test(handle: &Qemu, build: &str) -> Result<(), Box<dyn Error>> {
    // Upload the module
    let uploaded = format!(
        "/tmp/{:?}",
        Path::new(build).file_name().ok_or(BadFilePath)?
    );
    handle.transfer(&build, &uploaded)?;
    handle.runcmd(&format!("insmod {} ports=8000", uploaded))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Obtain the running config
    let opt = Opt::from_args();
    let config: Config = toml::from_slice(&read(opt.config)?)?;

    println!("{:#?}", config);

    // Init the cache
    let cache = Cache::new(&config.cache.expand_home()?);

    for mut kernel in config.kernels {
        // Download or retrieve cached items
        cache.get(&mut kernel)?;

        // Compile the module against the headers
        let build = ModuleBuilder::build(&config.module.name, &kernel)?;

        // Start qemu with the config
        let handle = Qemu::start(&kernel)?;

        test(&handle, &build).map_err(|e| println!("{:?}", e));

        std::thread::sleep(std::time::Duration::new(5, 0));

        handle.stop()?;
    }

    Ok(())
}
