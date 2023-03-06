use std::error::Error;
use std::process::Command;
use std::str::FromStr;

/// Wrapper to detect host SSH client version
pub struct SshVersion {
    version: f32,
}

impl SshVersion {
    /// Query the local ssh client version
    pub fn query() -> Result<Self, Box<dyn Error>> {
        let cmd = r"ssh -V 2>&1 | awk -F'[_,]' '{print $2+0}'";
        let output = Command::new("/bin/sh").arg("-c").arg(cmd).output()?;
        let version_str = std::str::from_utf8(&output.stdout)?.to_string();
        Ok(Self {
            version: f32::from_str(version_str.trim())?,
        })
    }

    pub fn version(&self) -> &f32 {
        &self.version
    }

    /// Test if the ssh version requires legacy support for max compatibility.
    /// This will return true if the host client is older than SSH 9.0.
    ///
    /// The scp client since 9.0 switches from using the legacy scp/rcp protocol
    /// to using the SFTP protocol by default. Reducing compatibility without -O.
    pub fn is_legacy(&self) -> bool {
        self.version <= 8.9
    }
}
