use crate::errors::NixModuleError::*;
use crate::KConfig;
use std::error::Error;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

pub struct Qemu {
    handle: Child,
    sshkey: String,
}

impl Qemu {
    /// Start Qemu with the provided configuration
    pub fn start(kernel: &KConfig) -> Result<Self, Box<dyn Error>> {
        let mut qemu = Command::new(&kernel.runner);

        // Optional args
        match &kernel.runner_extra_args {
            Some(ref extra) => {
                qemu.args(extra);
            }
            _ => {}
        }

        // Kick of the process
        let res = Self {
            handle: qemu
                .args(["-m", "2G", "-smp", "2"])
                .args(["-kernel", &kernel.kernel])
                .args([
                    "-append",
                    "console=ttyS0 root=/dev/sda earlyprintk=serial net.ifnames=0 nokaslr",
                ])
                .args(["-drive", &format!("file={},format=raw", &kernel.disk.path)])
                .args([
                    "-net",
                    "user,host=10.0.2.10,hostfwd=tcp:127.0.0.1:10021-:22",
                ])
                .args(["-net", "nic,model=e1000"])
                .arg("-enable-kvm")
                .arg("-nographic")
                .args(["-pidfile", "vm.pid"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .or(Err(QemuError))?,
            sshkey: kernel.disk.sshkey.clone(),
            //config: kernel,
        };

        // Wait until boot is complete/ssh is open
        while let Err(inn) = TcpStream::connect("127.0.0.1:10021") {
            sleep(Duration::new(2, 0));
        }

        Ok(res)
    }

    pub fn runcmd(&self, cmd: &str) -> Result<(), Box<dyn Error>> {
        println!("Running {}", cmd);
        let res = Command::new("ssh")
            .args(["-i", &self.sshkey])
            .args(["-p", "10021"])
            .args(["-oStrictHostKeyChecking=no"])
            .arg("root@localhost")
            .arg(cmd)
            .output()?;

        match res.status.success() {
            true => Ok(()),
            false => {
                println!("{:?}", std::str::from_utf8(&res.stderr)?);
                Err(SshError.into())
            }
        }
    }

    /// Transfer a file into the running VM
    pub fn transfer(&self, local: &str, remote: &str) -> Result<(), Box<dyn Error>> {
        println!("Uploading {}", local);
        let res = Command::new("scp")
            .args(["-i", &self.sshkey])
            .args(["-P", "10021"])
            .args(["-oStrictHostKeyChecking=no"])
            .arg(local)
            .arg(format!("root@localhost:{}", remote))
            .output()?;

        match res.status.success() {
            true => Ok(()),
            false => {
                println!("{:?}", std::str::from_utf8(&res.stderr)?);
                Err(SshError.into())
            }
        }
    }

    /// Stop the background qemu instance
    pub fn stop(mut self) -> Result<(), Box<dyn Error>> {
        self.handle.kill()?;
        Ok(())
    }
}
