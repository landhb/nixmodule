use home_dir::HomeDirExt;
use serde::Deserialize;
use std::error::Error;
use std::fs::{read, File};
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;

mod cache;
use cache::Cache;

mod errors;

#[derive(StructOpt, Debug)]
#[structopt(name = "nixmodule")]
struct Opt {
    #[structopt(
        short = "c",
        long = "config",
        default_value = "~/.config/nixmodule/config.toml"
    )]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    cache: PathBuf,
    kernels: Vec<KConfig>,
}

#[derive(Debug, Deserialize)]
struct KConfig {
    version: String,
    url_base: String,
    headers: String,
    kernel: String,
    disk: String,
    runner: String,
}

fn start_qemu(kernel: KConfig) {}

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

        // Start qemu with the local paths
        start_qemu(kernel);
    }

    Ok(())
}
