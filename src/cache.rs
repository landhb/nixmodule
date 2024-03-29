use colored::*;
use reqwest::blocking::Client;
use reqwest::Url;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::errors::NixModuleError::*;
use crate::KConfig;

// Archive
use flate2::read::GzDecoder;
use tar::Archive;

/// Cache of kernel boot files
pub struct Cache {
    dir: PathBuf,
}

#[derive(Debug)]
enum ArchiveType {
    TarGz,
    TarXz,
    TarBz2,
}

impl Cache {
    /// Initialize the cache
    pub fn new<T: AsRef<OsStr> + ?Sized>(cache: &T) -> Self {
        let cache = Path::new(&cache);
        if !cache.exists() {
            fs::create_dir_all(cache.join("downloads")).unwrap();
            fs::create_dir_all(cache.join("cache").join("images")).unwrap();
        }
        Self {
            dir: cache.to_path_buf(),
        }
    }

    /// Retrieve a path from the cache.
    ///
    /// This initiates a download if the file isn't present
    pub fn get(&self, kernel: &mut KConfig) -> Result<(), Box<dyn Error>> {
        log_status!("Checking artifacts for Linux Kernel {}", kernel.version);

        // The cache folder for this KConfig
        let cache_dir = self.dir.as_path().join("cache").join(&kernel.version);
        let images_dir = self.dir.as_path().join("cache").join("images");

        // Get headers
        let headers_url = format!("{}/{}", kernel.url_base, kernel.headers);
        let headers_cpath = cache_dir.join("headers");
        let headers_dpath = self.download(&headers_url, &headers_cpath)?;
        self.check_local(&headers_dpath, &headers_cpath)?;

        // Get bzImage
        let kernel_url = format!("{}/{}", kernel.url_base, kernel.kernel);
        let kernel_cpath = cache_dir.join(Path::new(&kernel.kernel).file_name().unwrap());
        let kernel_dpath = self.download(&kernel_url, &kernel_cpath)?;
        self.check_local(&kernel_dpath, &kernel_cpath)?;

        // Get disk image
        let disk_url = format!("{}/{}", kernel.disk.url_base, kernel.disk.path);
        let disk_cpath = images_dir.join(Path::new(&kernel.disk.path).file_name().unwrap());
        let disk_dpath = self.download(&disk_url, &disk_cpath)?;
        self.check_local(&disk_dpath, &disk_cpath)?;

        // Optional initrd
        if let Some(ref path) = kernel.disk.initrd {
            let initrd_url = format!("{}/{}", kernel.disk.url_base, path);
            let initrd_cpath = images_dir.join(Path::new(&path).file_name().unwrap());
            let initrd_dpath = self.download(&initrd_url, &initrd_cpath)?;
            self.check_local(&initrd_dpath, &initrd_cpath)?;
            kernel.disk.initrd = Some(
                initrd_cpath
                    .into_os_string()
                    .into_string()
                    .or(Err(BadFilePath))?,
            );
        }

        // Get ssh key
        let key_url = format!("{}/{}", kernel.disk.url_base, kernel.disk.sshkey);
        let key_cpath = images_dir.join(Path::new(&kernel.disk.sshkey).file_name().unwrap());
        let key_dpath = self.download(&key_url, &key_cpath)?;
        self.check_local(&key_dpath, &key_cpath)?;

        // Update the local paths
        kernel.headers = headers_cpath
            .into_os_string()
            .into_string()
            .or(Err(BadFilePath))?;
        kernel.kernel = kernel_cpath
            .into_os_string()
            .into_string()
            .or(Err(BadFilePath))?;
        kernel.disk.path = disk_cpath
            .into_os_string()
            .into_string()
            .or(Err(BadFilePath))?;
        kernel.disk.sshkey = key_cpath
            .into_os_string()
            .into_string()
            .or(Err(BadFilePath))?;

        Ok(())
    }

    /// Checks the cache path, or unpacks an existing download
    fn check_local(&self, dpath: &PathBuf, cpath: &PathBuf) -> Result<(), Box<dyn Error>> {
        // This response is already cached
        if cpath.as_path().exists() {
            return Ok(());
        }

        // If the file is an archive, unpack it first
        match dpath.extension() {
            Some(ext) if self.is_archive(ext).is_some() => {
                self.unpack(cpath, dpath, self.is_archive(ext).unwrap())?;
            }
            _ => {
                fs::rename(dpath, cpath)?;
                fs::set_permissions(cpath, Permissions::from_mode(0o600))?;
            }
        }
        Ok(())
    }

    /// Either performs a download or skips the request if the file
    /// already exists in $CACHE/downloads
    fn download<P: AsRef<Path>>(&self, uri: &str, cpath: &P) -> Result<PathBuf, Box<dyn Error>> {
        // Verify download isn't cached
        let url = Url::parse(uri)?;
        let fname = url
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        // Depending on type, determine the download location
        let fname = self.dir.as_path().join("downloads").join(fname);

        // This response is already downloaded
        if cpath.as_ref().exists() || fname.exists() {
            return Ok(fname);
        }

        log_status!("Downloading {}", uri);

        // Actually perform the request
        let client = Client::new();
        let mut response = client.get(uri).send()?;

        // Check result
        if !response.status().is_success() {
            return Err(format!("{} not found", uri).into());
        }

        // Create the outfile
        let mut outfile = File::create(&fname)?;
        response.copy_to(&mut outfile)?;
        Ok(fname)
    }

    /// Internal archive types
    fn is_archive(&self, ext: &OsStr) -> Option<ArchiveType> {
        match ext.to_str() {
            Some("xz") => Some(ArchiveType::TarXz),
            Some("gz") => Some(ArchiveType::TarGz),
            Some("bz2") => Some(ArchiveType::TarBz2),
            _ => None,
        }
    }

    /// Internal unarchiving code
    fn unpack(
        &self,
        outdir: &PathBuf,
        file: &PathBuf,
        atype: ArchiveType,
    ) -> Result<(), Box<dyn Error>> {
        match atype {
            ArchiveType::TarGz => {
                let tar_gz = File::open(file)?;
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                archive.unpack(outdir)?;
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}
