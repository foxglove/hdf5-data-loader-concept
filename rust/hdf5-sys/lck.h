// Define some things that are missing from WASI

#ifndef F_RDLCK
/* For posix fcntl() and `l_type' field of a `struct flock' for lockf().  */
# define F_RDLCK		0	/* Read lock.  */
# define F_WRLCK		1	/* Write lock.  */
# define F_UNLCK		2	/* Remove lock.  */
#endif


/* For old implementation of BSD flock.  */
#ifndef F_EXLCK
# define F_EXLCK		4	/* or 3 */
# define F_SHLCK		8	/* or 4 */
#endif

#  define F_GETLK	5	/* Get record locking info.  */
#  define F_SETLK	6	/* Set record locking info (non-blocking).  */
#  define F_SETLKW	7	/* Set record locking info (blocking).  */

