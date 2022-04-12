
# NixModule

Automatically test out-of-tree linux kernel modules across multiple kernel versions. The provided images all have KASAN, to aide your development.

Example running it against a single kernel:

<p align="center">
  <img width="460" height="300" src="img/filter.png">
</p>

Results table for multiple kernels:

<p align="center">
  <img width="460" height="300" src="img/results.png">
</p>

- [Pre-Built Kernels](#pre-built-kernels)  
- [Pre-Built Disk Images](#pre-built-disks)  
- [Using Other Kernels](#using-other-kernels)  
- [Using Other Disk Images](#using-other-disks)  

## Pre-Built Kernels <a name="pre-built-kernels"/>

| Version | BzImage   | Headers |
| :---:   | :---:     | :---:   |
| 5.17.2  | [bZimage](https://files.sboc.dev/linux-kernels/bzImage-5.17.2) | [Headers](https://files.sboc.dev/linux-headers/linux-5.17.2-headers.tar.gz)| 
| 5.8.9   | [bZimage](https://files.sboc.dev/linux-kernels/bzImage-5.8.9)  | [Headers](https://files.sboc.dev/linux-headers/linux-5.8.9-headers.tar.gz)| 
| 5.4.188 | [bZimage](https://files.sboc.dev/linux-kernels/bzImage-5.4.188)| [Headers](https://files.sboc.dev/linux-headers/linux-5.4.188-headers.tar.gz)| 
| 4.19.237| [bZimage](https://files.sboc.dev/linux-kernels/bzImage-4.19.237) | [Headers](https://files.sboc.dev/linux-headers/linux-4.19.237-headers.tar.gz)| 
| 4.14.275| [bZimage](https://files.sboc.dev/linux-kernels/bzImage-4.14.275) | [Headers](https://files.sboc.dev/linux-headers/linux-4.14.275-headers.tar.gz)| 
| 4.9.309| [bZimage](https://files.sboc.dev/linux-kernels/bzImage-4.9.309) | [Headers](https://files.sboc.dev/linux-headers/linux-4.4.309-headers.tar.gz)| 
| 4.1.52| [bZimage](https://files.sboc.dev/linux-kernels/bzImage-4.1.52) | [Headers](https://files.sboc.dev/linux-headers/linux-4.1.52-headers.tar.gz)| 

## Pre-Built Disk Images <a name="pre-built-disks"/>

| Name    | Link      | SSH Key |
| :---:   | :---:     | :---:   |
| Syzcaller Debian Stretch   | [Image](https://files.sboc.dev/images/stretch/stretch.img)  | [Key](https://files.sboc.dev/images/stretch/stretch.id_rsa)  

## Using Other Kernels <a name="using-other-kernels"/>

Use the packing script

```sh
KERNEL=4.14.275 ./scripts/package.sh
```

This builds the required `bzImage` and an archive `linux-$VERSION-headers.tar.gz` containing the headers/module info required to build an out-of-tree kernel module.

Then add the new kernel to your configuration file `nixmodule-config.toml`:

```toml
[[kernels]]
version = "4.19.237"
url_base = "https://files.sboc.dev"
headers = "linux-headers/linux-4.19.237-headers.tar.gz" 
kernel = "linux-kernels/bzImage-linux-4.19.237"
runner = "qemu-system-x86_64"

[kernels.disk]
name = "stretch"
url_base = "https://files.sboc.dev"
path = "images/stretch/stretch.img"
sshkey = "images/stretch/stretch.id_rsa"
```


## Using Other Disk Images <a name="using-other-disks"/>

Fill out the `[kernels.disk]` entry for the kernel you'd like to use the new disk with:

```toml
[kernels.disk]
name = "stretch"
url_base = "https://files.sboc.dev"
path = "images/stretch/stretch.img"
sshkey = "images/stretch/stretch.id_rsa"
boot = "/dev/sda"
```

Boot should contain the partition to boot from. This is passed directly to qemu to append as kernel arugments as:

```
-append "console=ttyS0 root=$BOOT earlyprintk=serial net.ifnames=0 nokaslr"
```