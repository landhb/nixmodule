use home_dir::HomeDirExt;
use serde::Deserialize;
use std::error::Error;
use std::fs::read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use structopt::StructOpt;

mod cache;
use cache::Cache;

mod errors;
use errors::NixModuleError;

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
    runner: String,
    runner_extra_args: Option<Vec<String>>,
}

fn start_qemu(kernel: KConfig) -> Result<Child, Box<dyn Error>> {
    let mut qemu = Command::new(&kernel.runner);

    // Optional args
    match kernel.runner_extra_args {
        Some(extra) => {
            qemu.args(extra);
        }
        _ => {}
    }

    // Kick of the process
    qemu.args(["-m", "2G", "-smp", "2"])
        .args(["-kernel", &kernel.kernel])
        .args([
            "-append",
            "console=ttyS0 root=/dev/sda earlyprintk=serial net.ifnames=0 nokaslr",
        ])
        .args(["-drive", &format!("file={},format=raw", &kernel.disk)])
        .args([
            "-net",
            "user,host=10.0.2.10,hostfwd=tcp:127.0.0.1:10021-:22",
        ])
        .args(["-net", "nic,model=e1000"])
        .arg("-enable-kvm")
        .arg("-nographic")
        .args(["-pidfile", "vm.pid"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .or(Err(NixModuleError::QemuError.into()))
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

        // Start qemu with the local paths
        let mut handle = start_qemu(kernel)?;

        std::thread::sleep(std::time::Duration::new(100, 0));

        handle.kill().unwrap();
    }

    Ok(())
}
