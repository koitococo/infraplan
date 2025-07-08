#include "utils.hpp"
#include <stdlib.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/reboot.h>
#include <linux/reboot.h>

long kexec_file_load(int kernel_fd, int initrd_fd, unsigned long cmdline_len, const char *cmdline, unsigned long flags) {
  return syscall(SYS_kexec_file_load, kernel_fd, initrd_fd, cmdline_len, cmdline, flags);
}

int kexec_reboot() {
  return reboot(LINUX_REBOOT_CMD_KEXEC);
}