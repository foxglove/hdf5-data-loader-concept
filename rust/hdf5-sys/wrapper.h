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

#include "hdf5.h";

size_t H5Z_lzf_filter(unsigned int flags, size_t cd_nelmts, const unsigned int cd_values[],
                             size_t nbytes, size_t *buf_size, void **buf);

herr_t        H5Z_lzf_set_local(hid_t dcpl, hid_t type, hid_t space);
