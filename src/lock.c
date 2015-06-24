#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>

int c_lock(int fd, int should_block, int is_write_lock) {
  if (fd < 0) {
    return EBADF;
  }

  struct flock fl;

  fl.l_type   = is_write_lock 
                ? F_WRLCK 
                : F_RDLCK;
  fl.l_whence = SEEK_SET;
  fl.l_start  = 0;
  fl.l_len    = 0;
  if (fcntl(fd, should_block
                ? F_SETLKW 
                : F_SETLK, &fl) == -1) {
    return errno;
  }

  return 0;
}

int c_unlock(int fd) {
  struct flock fl;

  if (fd < 0) {
    return EBADF;
  }

  fl.l_type = F_UNLCK;
  fl.l_whence = SEEK_SET;
  fl.l_start = 0;
  fl.l_len = 0;

  if (fcntl(fd, F_SETLK, &fl) == -1) {
    return errno;
  }

  return 0;
}
