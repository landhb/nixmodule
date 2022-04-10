use home_dir::HomeDirExt;
use serde::Deserialize;
use std::error::Error;
use std::fs::read;
use std::path::PathBuf;
use structopt::StructOpt;

mod cache;
use cache::Cache;

mod errors;
use errors::NixModuleError;

mod qemu;
use qemu::Qemu;

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
pub struct KConfig {
    version: String,
    url_base: String,
    headers: String,
    kernel: String,
    disk: String,
    sshkey: String,
    runner: String,
    runner_extra_args: Option<Vec<String>>,
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

        // Start qemu with the local paths
        let handle = Qemu::start(kernel)?;

        // Begin testing
        handle.transfer("/etc/passwd", "/tmp/passwd")?;

        std::thread::sleep(std::time::Duration::new(100, 0));

        handle.stop()?;
    }

    Ok(())
}
