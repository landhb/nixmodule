
# NixModule



## Pre-Built Kernels

| Version | BzImage | Headers |

## Pre-Built Disk Images



## Using Other Kernels

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

## Using Other Disk Images

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