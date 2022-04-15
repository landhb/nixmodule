CONFIG_MODULE_SIG=n

# Updated as example-$VERSION during tests
TARGET=example
obj-m += $(TARGET).o
$(TARGET)-objs := src/example.o

# Overriden with kernel version during tests
KERNEL := /lib/modules/$(shell uname -r)/build
KDIR := $(KERNEL)
PWD := $(shell pwd)

debug:
	$(MAKE) -C $(KDIR) M=$(PWD) modules
	rm -f *.mod* *.o .example* *.cmd

release:
	$(MAKE) -C $(KDIR) M=$(PWD) modules
	rm -f *.mod* *.o .example* *.cmd
	
clean:
	$(MAKE) -C $(KERNEL) M=$(PWD) clean