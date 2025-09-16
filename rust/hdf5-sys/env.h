inline void *dlopen(const char*, int) {}
inline char *dlerror(void) {}
inline void *dlsym(void *__restrict, const char *__restrict) {}
inline int dlclose(void *) {}

#define FE_INVALID          0x0001
