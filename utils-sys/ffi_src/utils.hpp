long kexec_file_load(int kernel_fd, int initrd_fd, unsigned long cmdline_len, const char *cmdline, unsigned long flags);

int kexec_reboot();