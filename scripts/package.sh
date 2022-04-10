#!/bin/bash
set -e
trap 'previous_command=$this_command; this_command=$BASH_COMMAND' DEBUG
trap 'echo FAILED COMMAND: $previous_command' EXIT

: ${KERNEL:=linux-4.19.237}
BUILD_DIR=/tmp/package$KERNEL
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

mkdir $BUILD_DIR || true

# Download source
wget -nc -O $BUILD_DIR/$KERNEL.tar.xz https://cdn.kernel.org/pub/linux/kernel/v4.x/$KERNEL.tar.xz || echo "$KERNEL.tar.xz exists"

# Extract everything if not extracted
pushd $BUILD_DIR
for f in $BUILD_DIR/*.tar*; do if [[ ! -d $(basename $f | sed 's/\(.*\)\..*/\1/' | xargs basename -s .tar) ]]; then tar xfk $f; fi; done
popd

# Build the kernel image
pushd $BUILD_DIR/$KERNEL
make defconfig
make kvmconfig | make kvm_guest.config
echo "# Coverage collection.
CONFIG_KCOV=y

# Debug info for symbolization.
CONFIG_DEBUG_INFO=y

# Memory bug detector
CONFIG_KASAN=y
CONFIG_KASAN_INLINE=y

# Required for Debian Stretch
CONFIG_CONFIGFS_FS=y
CONFIG_SECURITYFS=y" >> .config
make olddefconfig
make -j`nproc`
cp arch/x86_64/boot/bzImage $BUILD_DIR/bzImage-$KERNEL
popd

# Package the headers and module information
pushd $BUILD_DIR
KCONFIG_CONFIG=.config SRCARCH=x86 objtree=$BUILD_DIR/$KERNEL/ srctree=$BUILD_DIR/$KERNEL/ $SCRIPT_DIR/module_headers_install.sh
cd linux-modules-headers
tar -czvf ../$KERNEL-headers.tar.gz *
popd


echo "Package is in $BUILD_DIR"