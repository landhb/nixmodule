
## How to Use

Use the packing script

```sh
KERNEL=linux-4.14.275 ./scripts/package.sh
```

Build the kernel by following the instructions in BUILDING_IMAGES.md



then run this to create the necessary header export:

```sh
KCONFIG_CONFIG=.config SRCARCH=x86 objtree=linux-5.8.9/ srctree=linux-5.8.9/ ./module_headers_install.sh
```

then tar it for future use:

```sh
cd linux-modules-headers
tar -czvf ../linux-5.8.9-headers.tar.gz *
```
