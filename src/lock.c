#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>

struct result {
  int fd;
  int error;
};

struct result c_lock(const char* pathname, int should_block) {
  struct result my_result;                     
                                               
  int fd = open(                               
    pathname,                                  
    (O_WRONLY | O_CREAT),                      
    (S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH)    
  );                                           
                                               
  if (fd == -1) {                              
    my_result.fd    = -1;                      
    my_result.error = errno;                   
    return my_result;                          
  }                                            
                                               
  struct flock fl;                             
  fl.l_type   = F_WRLCK;                       
  fl.l_whence = SEEK_SET;                      
  fl.l_start  = 0;                             
  fl.l_len    = 0;                             
                                               
  if (fcntl(fd, should_block
                ? F_SETLKW 
                : F_SETLK, &fl) == -1) {             
    my_result.fd    = -1;                      
    my_result.error = errno;                   
    close(fd);                                 
    return my_result;                          
  }                                            
                                               
  my_result.fd    = fd;                        
  my_result.error = 0;                         
  return my_result;
}                            

struct result c_unlock(int fd) {
  struct result my_result;

  if (close(fd) == -1) {
    my_result.fd    = -1;
    my_result.error = errno;
    return my_result;
  }

  my_result.fd    = -1;
  my_result.error = 0;
  return my_result;
}
