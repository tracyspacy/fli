use crate::{
    errors::{
        FliError::{
            DirFd, FStatAt, Getgrgid, Getpwuid, LocalTime, NoSuchFileOrDir, OpenDirError, ReadLink,
            StrFTime, WrongEntryType,
        },
        FliResult,
    },
    output_config::View,
};
use core::ffi::c_char;
/*
*opendir()* function:
opens a directory stream corresponding to
the directory name, and returns a pointer to the directory stream.
The stream is positioned at the first entry in the directory.
*/

//helper to check type prior
//
pub fn is_dir(path: *const c_char) -> FliResult<bool> {
    let mut stat_buf = core::mem::MaybeUninit::<libc::stat>::uninit();
    if unsafe { libc::stat(path, stat_buf.as_mut_ptr()) } != 0 {
        Err(NoSuchFileOrDir)
    } else {
        let mode = unsafe { stat_buf.assume_init().st_mode };
        Ok(mode & libc::S_IFMT == libc::S_IFDIR)
    }
}

pub struct OpenDir {
    dir: *mut libc::DIR,
    fd: i32, // file descriptor
}

/*
opendir()
The opendir() return a pointer to the directory stream.  On error, NULL is returned
dirfd()
On success, dirfd() returns a file descriptor (a nonnegative integer).  On error, -1 is returned
*/

impl OpenDir {
    // how correct it is , if default is just same folder
    pub fn new(path: *const c_char) -> FliResult<Self> {
        let dir = unsafe { libc::opendir(path) };
        if dir.is_null() {
            return Err(OpenDirError);
        }
        let fd = unsafe { libc::dirfd(dir) };
        if fd.is_negative() {
            // explicitely closing dir
            // probably overkill,
            unsafe { libc::closedir(dir) };
            return Err(DirFd);
        }
        Ok(Self { dir, fd })
    }
}

impl Drop for OpenDir {
    fn drop(&mut self) {
        unsafe {
            libc::closedir(self.dir);
        }
    }
}

/*
The readdir() function returns a pointer to a dirent structure
       representing the next directory entry in the directory stream
       pointed to by dirp.  It returns NULL on reaching the end of the
       directory stream or if an error occurred.
*/

/*
https://www.man7.org/linux/man-pages/man3/readdir.3.html has no d_namelen but has d_type

pub struct dirent {
        pub d_ino: crate::ino_t,
        pub d_offset: off_t,
        pub d_reclen: c_short,
        pub d_namelen: c_short, // do it have namelen for all unix?
        pub d_name: [c_char; 1], // flex array
    }
*/

// probably need just d_name + d_namelen + d_type
pub struct DirEntry {
    dirfd: i32,
    dirent: *mut libc::dirent,
}
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum EntryType {
    Directory = 0,
    RegularFile = 1,
    SymLink = 2,
    Unknown = 3,
    Other = 4,
}

// emoji&text: each with trailing whitespace except for other
// color: ansi color codes https://gist.github.com/JBlond/2fea43a3049b38287e5e9cefc87b2124 , unknown and other are empty
const ETFMT: &[u8] =
    b"\xF0\x9F\x97\x82\xEF\xB8\x8F \xF0\x9F\x93\x84 \xF0\x9F\x94\x97 ?  </DIR> <FILE> <LINK> \x1B[0;34m \x1B[0;32m \x1B[0;36m";

// check carefully if overlaps
const ETOFFSET: [(u8, u8); 15] = [
    (0, 8),  // folder emoji
    (8, 5),  // file emoji
    (13, 5), // link emoji
    (17, 3), // unknown
    (20, 0), // empty
    (21, 7), // </DIR>+whitespace
    (28, 7), // <FILE>+whitespace
    (35, 7), // <LINK>+whitespace
    (17, 3), // reuse of unknown
    (20, 0), // reuse of empty
    (42, 7), // blue for dir
    (50, 7), // green for file
    (58, 7), // cyan for link
    (20, 0), // empty for unknown
    (20, 0), // empty for other
];

impl EntryType {
    pub fn view_fmt(self, adj: View) -> &'static [u8] {
        let idx = self as usize + adj as usize;
        let (s, e) = ETOFFSET[idx];
        &ETFMT[s as usize..s as usize + e as usize]
    }
    pub fn is_symlink(&self) -> bool {
        matches!(self, EntryType::SymLink)
    }
}

impl From<u8> for EntryType {
    fn from(d_type: u8) -> Self {
        match d_type {
            libc::DT_DIR => Self::Directory,
            libc::DT_REG => Self::RegularFile,
            libc::DT_LNK => Self::SymLink,
            libc::DT_UNKNOWN => Self::Unknown,
            _ => Self::Other,
        }
    }
}

impl DirEntry {
    pub fn name(&self) -> &core::ffi::CStr {
        unsafe { core::ffi::CStr::from_ptr((*self.dirent).d_name.as_ptr()) }
    }
    pub fn entry_type(&self) -> EntryType {
        EntryType::from(unsafe { (*self.dirent).d_type })
    }

    // https://www.man7.org/linux/man-pages/man2/readlink.2.html
    // returns number of bytes placed in buffer, so len
    pub fn sym_link_with_value(&self, name: &core::ffi::CStr) -> FliResult<([u8; 255], usize)> {
        if !self.entry_type().is_symlink() {
            return Err(WrongEntryType);
        }
        //If the returned value equals bufsiz, then truncation may have occurred.
        // buffer size, maybe can use smaller
        let mut buffer = [0u8; 255];
        let path_len = unsafe {
            libc::readlinkat(
                self.dirfd,
                name.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_char,
                buffer.len(),
            )
        };
        if path_len == -1 {
            return Err(ReadLink);
        }
        Ok((buffer, path_len as usize))
    }
}

impl Iterator for OpenDir {
    type Item = DirEntry;
    fn next(&mut self) -> Option<DirEntry> {
        loop {
            let dirent = unsafe { libc::readdir(self.dir) };
            if dirent.is_null() {
                return None;
            }
            let name = unsafe { core::ffi::CStr::from_ptr((*dirent).d_name.as_ptr()) };
            //expensive . find a better way
            let name_bytes = name.to_bytes();

            if name_bytes == b"." || name_bytes == b".." {
                continue;
            }
            return Some(DirEntry {
                dirfd: self.fd,
                dirent,
            });
        }
    }
}

// https://man7.org/linux/man-pages/man3/stat.3type.html
/*
           dev_t      st_dev;      /* ID of device containing file */
           ino_t      st_ino;      /* Inode number */
           mode_t     st_mode;     /* File type and mode */
           nlink_t    st_nlink;    /* Number of hard links */
           uid_t      st_uid;      /* User ID of owner */
           gid_t      st_gid;      /* Group ID of owner */
           dev_t      st_rdev;     /* Device ID (if special file) */
           off_t      st_size;     /* Total size, in bytes */
           blksize_t  st_blksize;  /* Block size for filesystem I/O */
           blkcnt_t   st_blocks;   /* Number of 512 B blocks allocated */
*/

pub struct Metadata(libc::stat);
#[allow(unused)]
impl Metadata {
    pub fn new(entry: &DirEntry) -> FliResult<Self> {
        let mut stat_buf = core::mem::MaybeUninit::<libc::stat>::uninit();
        let name = unsafe { (*entry.dirent).d_name };
        //  https://man.freebsd.org/cgi/man.cgi?query=fstatat&sektion=2&n=1
        // can return -1 ie error
        // https://www.man7.org/linux/man-pages/man2/access.2.html regarding AT_SYMLINK_NOFOLLOW
        let s = unsafe {
            libc::fstatat(
                entry.dirfd,
                name.as_ptr(),
                stat_buf.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if s == 0 {
            Ok(Self(unsafe { stat_buf.assume_init() }))
        } else {
            Err(FStatAt)
        }
    }
    // to make it cross compile eg arm-unknown-linux-gnueabihf size is i32 not i64

    pub fn size(&self) -> usize {
        self.0.st_size as usize
    }
    pub fn n_link(&self) -> usize {
        self.0.st_nlink as usize
    }
    pub fn st_mtime(&self) -> usize {
        self.0.st_mtime as usize
    }
    // need to format properly, not as timestamp
    // clippy!! where are you my Lord and Savior?!
    pub fn last_modified_fmt(&self) -> FliResult<[u8; 17]> {
        //example output 2026-06-06 16:11 ie 16 bytes + null byte
        // need to consider this then pushing to buffer
        let mut char_buf: [u8; 17] = [0u8; 17];
        let mut tm = core::mem::MaybeUninit::<libc::tm>::zeroed();
        unsafe {
            let mtime = &self.0.st_mtime as *const libc::time_t;
            // https://linux.die.net/man/3/localtime_r
            // can return null
            let tm_ptr = libc::localtime_r(mtime, tm.as_mut_ptr());
            if tm_ptr.is_null() {
                return Err(LocalTime);
            }
            let tm_ref = tm.assume_init_ref();
            //https://www.man7.org/linux/man-pages/man3/strftime.3.html
            // ISO8601 format YYYY-MM-DD
            let fmt = b"%Y-%m-%d %H:%M\0";
            let writer = libc::strftime(
                char_buf.as_mut_ptr() as *mut libc::c_char,
                char_buf.len(),
                fmt.as_ptr() as *const libc::c_char,
                tm_ref,
            );
            // can be 0, so size is exceed the buffer size
            if writer == 0 {
                return Err(StrFTime);
            }
        };
        Ok(char_buf)
    }

    pub fn get_pw_uid(&self) -> usize {
        self.0.st_uid as usize
    }

    // https://www.man7.org/linux/man-pages/man3/getpwuid.3p.html
    pub fn user_bytes(&self) -> FliResult<&[u8]> {
        let pw = unsafe { libc::getpwuid(self.0.st_uid) };
        // can return null pointer
        if pw.is_null() {
            Err(Getpwuid)
        } else {
            Ok(unsafe { core::ffi::CStr::from_ptr((*pw).pw_name).to_bytes() })
        }
    }

    pub fn get_gr_gid(&self) -> usize {
        self.0.st_gid as usize
    }

    pub fn group_bytes(&self) -> FliResult<&[u8]> {
        let gr = unsafe { libc::getgrgid(self.0.st_gid) };
        // can return null pointer
        if gr.is_null() {
            Err(Getgrgid)
        } else {
            Ok(unsafe { core::ffi::CStr::from_ptr((*gr).gr_name).to_bytes() })
        }
    }

    // need to fetch all needed params
    // https://man7.org/linux/man-pages/man2/chmod.2.html
    // https://jameshfisher.com/2017/02/24/what-is-mode_t/
    pub fn mode_bytes(&self) -> [u8; 9] {
        let mode_digit = self.0.st_mode;
        let fmt = b"rwxrwxrwx";
        //will end with zero
        let mut buf = [b'-'; 9];
        for i in 0..9 {
            if mode_digit & (1 << (8 - i)) != 0 {
                buf[i] = fmt[i]
            };
        }
        buf
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::output_config::View;
    // each with trailing whitespace except for other
    const FMT: [(EntryType, View, &[u8]); 15] = [
        (
            EntryType::Directory,
            View::Emoji,
            b"\xF0\x9F\x97\x82\xEF\xB8\x8F ",
        ),
        (EntryType::RegularFile, View::Emoji, b"\xF0\x9F\x93\x84 "),
        (EntryType::SymLink, View::Emoji, b"\xF0\x9F\x94\x97 "),
        (EntryType::Unknown, View::Emoji, b" ? "),
        (EntryType::Other, View::Emoji, b""),
        (EntryType::Directory, View::Text, b"</DIR> "),
        (EntryType::RegularFile, View::Text, b"<FILE> "),
        (EntryType::SymLink, View::Text, b"<LINK> "),
        (EntryType::Unknown, View::Text, b" ? "),
        (EntryType::Other, View::Text, b""),
        (EntryType::Directory, View::Color, b"\x1B[0;34m"),
        (EntryType::RegularFile, View::Color, b"\x1B[0;32m"),
        (EntryType::SymLink, View::Color, b"\x1B[0;36m"),
        (EntryType::Unknown, View::Color, b""),
        (EntryType::Other, View::Color, b""),
    ];

    #[test]
    fn et_offset_bounds_test() {
        ETOFFSET.iter().for_each(|&(s, e)| {
            let end = s as usize + e as usize;
            assert!(end <= ETFMT.len(), "exceeds len")
        });
    }

    #[test]
    fn et_view_test() {
        FMT.iter().for_each(|&(entry_type, view, exp)| {
            let fmt = entry_type.view_fmt(view);
            assert_eq!(
                fmt, exp,
                "entry type {:?} view: {:?}",
                entry_type as u8, view as u8
            )
        });
    }
}
