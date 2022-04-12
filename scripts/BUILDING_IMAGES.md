### Installing QEMU

First, you need to install QEMU and other tools necessary to build the kernel and run it. This blog lists the libraries that need to be installed:

```sh
sudo apt-get update
sudo apt-get install git fakeroot build-essential ncurses-dev xz-utils libssl-dev bc flex libelf-dev bison qemu-system-x86
```

### Getting kernel source

The kernel source code is available at a bunch of places. But the best way to get it (if you don’t need the entire git history that comes with Torvalds branch) is the kernel.org website. All Linux versions are available to download via HTTP from there: https://mirrors.edge.kernel.org/pub/linux/kernel/.

I will select Linux 5.10.54, and download it to the machine via wget:

wget https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.10.54.tar.xz
tar xvf linux-5.10.54.tar.xz
cd linux-5.10.54

### Building the kernel

Here is another advantage of building the kernel for QEMU, instead of a full Ubuntu virtual machine. The amount of time it takes to build is way shorter when preparing it for QEMU, because there are much fewer modules and features that need to be enabled compared to Ubuntu.

To prepare the kernel for building, we need to set up the .config file. Fortunately, this step is easy:

```sh
# from inside Linux-5.10.54 folder
make defconfig # creates a .config file
make kvmconfig # modifies .config to set up everything necessary for it to run on QEMU
# or make kvm_guest.config in more recent kernels
```

You can confirm it worked by cat .config | grep KVM:

```sh
CONFIG_KVM_GUEST=y
# CONFIG_KVM_DEBUG_FS is not set
CONFIG_HAVE_KVM=y
# CONFIG_KVM is not set
CONFIG_PTP_1588_CLOCK_KVM=y
```

The last thing, but which is not completely necessary, is to add some debug configurations to the Kernel, like KASAN, debug symbols, etc. More information can be found at syzkaller Github. To do it, open .config in your favorite text editor and append the following to the end:

```sh
# Coverage collection.
CONFIG_KCOV=y

# Debug info for symbolization.
CONFIG_DEBUG_INFO=y

# Memory bug detector
CONFIG_KASAN=y
CONFIG_KASAN_INLINE=y

# Required for Debian Stretch
CONFIG_CONFIGFS_FS=y
CONFIG_SECURITYFS=y
```

Finally, run make olddefconfig to regenerate the configurations with the necessary modifications that the previous lines introduced.

```sh
make olddefconfig
```

Finally, to start building the kernel, run make. You can also specify the number of threads to use during compilation. I recommend setting it to something between nproc and nproc*2, to get the best compilation speed.

```sh
make -j`nproc`
```

It could take a few dozens of minutes, so be patient. In the end, a bzImage file should have been created:

```sh
ls arch/x86_64/boot/bzImage
```

### Creating an image for the kernel

The kernel cannot boot without a filesystem. There are multiple ways to set up one, including initramfs and others, but I prefer to follow syzkaller guide again and use their script to set up a Debian-like environment which comes with a bunch of handy tools:

```sh
# from the source folder root
sudo apt-get install debootstrap
mkdir image && cd image
wget https://raw.githubusercontent.com/google/syzkaller/master/tools/create-image.sh -O create-image.sh
chmod +x create-image.sh
./create-image.sh
```

This step will also take a lot of time. I actually like to both compile and create the image at the same time, to save some time.

The result is a chroot folder is created, alongside an RSA key that will be used to ssh into QEMU when booted, and stretch.img, which is the actual file system.
Running the kernel

Finally, to run the kernel, I have a script run.sh that I like to use to make things easier:

```sh
qemu-system-x86_64 \
        -m 2G \
        -smp 2 \
        -kernel $1/arch/x86/boot/bzImage \
        -append "console=ttyS0 root=/dev/sda earlyprintk=serial net.ifnames=0 nokaslr" \
        -drive file=$2/stretch.img,format=raw \
        -net user,host=10.0.2.10,hostfwd=tcp:127.0.0.1:10021-:22 \
        -net nic,model=e1000 \
        -enable-kvm \
        -nographic \
        -pidfile vm.pid \
        2>&1 | tee vm.log
```

Put it in the linux-5.10.54 folder, and run it:

```sh
chmod +x run.sh
./run.sh . image/
```

If everything went right, you should now see QEMU running and a lot of kernel messages in your terminal. When prompted, the login is root, and there is no password.
Bonus: access via ssh

You can also access the QEMU machine via ssh:

```sh
ssh -i image/stretch.id_rsa -p 10021 -o "StrictHostKeyChecking no" root@localhost
```
