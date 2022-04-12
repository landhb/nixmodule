#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Bradley Landherr https://github.com/landhb");
MODULE_DESCRIPTION("Example for NixModule");

static int hello_init(void)
{
    printk(KERN_ALERT "Module loaded - Hello from the kernel.\n");
    return 0;
}

static void hello_exit(void)
{
    printk(KERN_ALERT "Goodbye, module unloaded\n");
}

// Register the initialization and exit functions
module_init(hello_init);
module_exit(hello_exit);