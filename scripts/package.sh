#!/bin/bash
set -e
trap 'previous_command=$this_command; this_command=$BASH_COMMAND' DEBUG
trap 'echo FAILED COMMAND: $previous_command' EXIT

: ${KERNEL:=4.19.237}
BUILD_DIR=/tmp/package-linux-$KERNEL
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

mkdir $BUILD_DIR || true

version_greater_equal()
{
    printf '%s\n%s\n' "$2" "$1" | sort --check=quiet --version-sort
}

if version_greater_equal $KERNEL 5.0 ; then
    URL="v5.x"
elif version_greater_equal $KERNEL 4.0 ; then
    URL="v4.x"
elif version_greater_equal $KERNEL 3.0 ; then
    URL="v3.x"
else
    echo "That kernel is pretty old....u sure?"
    exit 1
fi

# Download source
wget -nc -O $BUILD_DIR/$KERNEL.tar.xz https://cdn.kernel.org/pub/linux/kernel/$URL/linux-$KERNEL.tar.xz || echo "$KERNEL.tar.xz exists"

# Extract everything if not extracted
pushd $BUILD_DIR
for f in $BUILD_DIR/*.tar*; do if [[ ! -d $(basename $f | sed 's/\(.*\)\..*/\1/' | xargs basename -s .tar) ]]; then tar xfk $f; fi; done
popd

# Build the kernel image
pushd $BUILD_DIR/linux-$KERNEL
make defconfig
make kvmconfig | make kvm_guest.config
echo "
CONFIG_CRYPTO_RSA=y

# Coverage collection.
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
cp arch/x86_64/boot/bzImage $BUILD_DIR/bzImage-linux-$KERNEL
popd

# Package the headers and module information
pushd $BUILD_DIR
KCONFIG_CONFIG=.config SRCARCH=x86 objtree=$BUILD_DIR/linux-$KERNEL/ srctree=$BUILD_DIR/linux-$KERNEL/ $SCRIPT_DIR/module_headers_install.sh
cd linux-modules-headers
tar -czvf ../linux-$KERNEL-headers.tar.gz *
popd


echo "Package is in $BUILD_DIR" || true