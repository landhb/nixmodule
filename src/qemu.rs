use crate::errors::NixModuleError::*;
use crate::KConfig;
use rand::Rng;
use std::error::Error;
use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

pub struct Qemu {
    handle: Child,
    sshkey: String,
    sshport: String,
}

fn print_output(out: &str) {
    for line in out.split("\n") {
        println!("{}", line);
    }
}

impl Qemu {
    /// Start Qemu with the provided configuration
    pub fn start(kernel: &KConfig) -> Result<Self, Box<dyn Error>> {
        let mut qemu = Command::new(&kernel.runner);

        // Generate random high port for ssh
        let port: u16 = rand::thread_rng().gen_range(1025..=65535);

        // Optional args
        match &kernel.runner_extra_args {
            Some(ref extra) => {
                qemu.args(extra);
            }
            _ => {}
        }

        let fwd = format!("user,host=10.0.2.10,hostfwd=tcp:127.0.0.1:{}-:22", port);

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
                .args(["-net", &fwd])
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
            sshport: port.to_string(),
        };

        // Wait until boot is complete/ssh is open
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        while let Err(_) = TcpStream::connect(addr) {
            sleep(Duration::new(5, 0));
        }

        Ok(res)
    }

    pub fn runcmd(&self, cmd: &str) -> Result<(), Box<dyn Error>> {
        println!("Running {}", cmd);
        let res = Command::new("ssh")
            .args(["-i", &self.sshkey])
            .args(["-p", &self.sshport])
            .args(["-oStrictHostKeyChecking=no"])
            .arg("root@localhost")
            .arg(cmd)
            .output()?;

        match res.status.success() {
            true => Ok(()),
            false => {
                print_output(std::str::from_utf8(&res.stderr)?);
                print_output(std::str::from_utf8(&res.stdout)?);
                Err(SshError.into())
            }
        }
    }

    /// Transfer a file into the running VM
    pub fn transfer(&self, local: &str, remote: &str) -> Result<(), Box<dyn Error>> {
        println!("Uploading {}", local);
        let res = Command::new("scp")
            .args(["-i", &self.sshkey])
            .args(["-P", &self.sshport])
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
