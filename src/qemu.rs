use crate::errors::NixModuleError::*;
use crate::utils::print_output;
use crate::KConfig;
use colored::*;
use rand::Rng;
use std::error::Error;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Qemu {
    handle: Child,
    sshkey: String,
    sshport: String,
}

impl Qemu {
    /// Start Qemu with the provided configuration
    pub fn start(kernel: &KConfig, debug: bool) -> Result<Self, Box<dyn Error>> {
        let timeout = Duration::new(kernel.timeout.map_or(60, |v| v), 0);
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

        // KVM?
        if kernel.kvm {
            qemu.arg("-enable-kvm");
        }

        // Start gdbserver in debug mode
        if debug {
            qemu.arg("-s");
        }

        let fwd = format!("user,host=10.0.2.10,hostfwd=tcp:127.0.0.1:{}-:22", port);
        let bootargs = format!(
            "console=ttyS0 root={} earlyprintk=serial net.ifnames=0 nokaslr",
            kernel.disk.boot
        );

        // Kick of the process
        let res = Self {
            handle: qemu
                .args(["-m", "512M", "-smp", "2"])
                .args(["-kernel", &kernel.kernel])
                .args(["-append", &bootargs])
                .args(["-drive", &format!("file={},format=raw", &kernel.disk.path)])
                .args(["-net", &fwd])
                .args(["-net", "nic,model=e1000"])
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

        log_status!("Waiting for VM to boot...");
        if let Err(e) = res.wait_for_boot(port, timeout) {
            res.stop()?;
            return Err(e);
        }

        Ok(res)
    }

    /// hacky workaround to wait for boot to finish
    fn wait_for_boot(&self, port: u16, timeout: Duration) -> Result<(), Box<dyn Error>> {
        // Wait until boot is complete/port is open
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let start = Instant::now();
        while let Err(_) = TcpStream::connect_timeout(&addr, timeout) {
            if start.elapsed() > timeout {
                return Err(TimeoutError.into());
            }
            sleep(Duration::new(7, 0));
        }

        // Wait for ssh service to start
        let mut buf = [0u8; 8];
        let mut stream = TcpStream::connect_timeout(&addr, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        let start = Instant::now();
        while let Err(_) = stream.read(&mut buf) {
            if start.elapsed() > timeout {
                return Err(TimeoutError.into());
            }
            sleep(Duration::new(7, 0));
        }
        Ok(())
    }

    pub fn runcmd(&self, cmd: &str) -> Result<(), Box<dyn Error>> {
        log_status!("Running {}", cmd);
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
        log_status!("Uploading {}", local);
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

    /// Enter an interactive shell on the running VM
    /// does not return until the shell exits
    pub fn interact(&self) -> Result<(), Box<dyn Error>> {
        let mut session = Command::new("ssh")
            .args(["-i", &self.sshkey])
            .args(["-p", &self.sshport])
            .args(["-oStrictHostKeyChecking=no"])
            .arg("root@localhost")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .spawn()?;

        log_status!("Use 'target remote localhost:1234' to connect to the gdb server");
        match session.wait() {
            Ok(_status) => Ok(()),
            Err(e) => {
                println!("{:?}", e);
                Err(SshError.into())
            }
        }
    }

    /// Stop the background qemu instance
    pub fn stop(mut self) -> Result<(), Box<dyn Error>> {
        self.handle.kill()?;
        self.handle.wait()?;
        Ok(())
    }
}
