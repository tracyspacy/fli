#![no_std]
#![no_main]

// TODO:
// - split to mods - hard to read already
// - add tests

use core::ffi::c_char;
use libc::{STDOUT_FILENO, fstatat, opendir, write};
extern crate alloc;
use alloc::{str, vec::Vec};

const MAX_INT_LEN: usize = 20;
type IntBytes = [u8; MAX_INT_LEN];

#[global_allocator]
static ALLOCATOR: LibcAllocator = LibcAllocator;

struct LibcAllocator;

unsafe impl core::alloc::GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { libc::malloc(layout.size()) as *mut u8 }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        unsafe { libc::free(ptr as *mut _) }
    }
}

//4 kb sounds nice
const BUFFER_SIZE: usize = 4096;

// helper write
fn write_bytes(bytes: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, bytes.as_ptr() as *const _, bytes.len());
    }
}

// helper - primitive dgit counter
//helper
#[inline]
fn digit_count(mut n: usize) -> usize {
    if n < 10 {
        return 1;
    }
    if n < 100 {
        return 2;
    }
    if n < 1_000 {
        return 3;
    }
    if n < 10_000 {
        return 4;
    }
    if n < 100_000 {
        return 5;
    }
    if n < 1_000_000 {
        return 6;
    }
    if n < 10_000_000 {
        return 7;
    }
    if n < 100_000_000 {
        return 8;
    }
    // should be less than u32:max len
    if n < 1_000_000_000 {
        return 9;
    }

    let mut c = 10;
    while n >= 1_000_000_000 {
        n /= 10;
        c += 1;
    }
    c
}

// helper to align int in bytes array if max len 5 but int is 12 ie 2 [b' ',b' ',b' ',b'1',b'2']
fn align_int(buf: &mut IntBytes, digit: usize, width: usize) -> &[u8] {
    buf.fill(b' ');
    if digit == 0 {
        buf[width - 1] = b'0';
        return &buf[..width];
    }
    let mut tmp = digit;
    let mut pos = width;
    while tmp > 0 {
        pos -= 1;
        // b'0' = [48] + res of % till 9=> ascii numbers 0-9
        buf[pos] = b'0' + (tmp % 10) as u8;
        tmp /= 10;
    }
    buf[..pos].fill(b' ');
    &buf[..width]
}

// helper for natural cmp
// TODO:
// need to consider register! now Videos goes before fli
//
fn natural_cmp(left: &[u8], right: &[u8]) -> core::cmp::Ordering {
    let mut i = 0;
    let mut j = 0;
    while i < left.len() && j < right.len() {
        if left[i].is_ascii_digit() && right[j].is_ascii_digit() {
            let left_digit_start = i; // to jump later
            while i < left.len() && left[i].is_ascii_digit() {
                i += 1;
            }
            let right_digit_start = j; // to jump later
            while j < right.len() && right[j].is_ascii_digit() {
                j += 1;
            }
            let left_digit = &left[left_digit_start..i];
            let right_digit = &right[right_digit_start..j];
            let left_digit_len = left_digit.len();
            let right_digit_len = right_digit.len();
            if left_digit_len != right_digit_len {
                return left_digit_len.cmp(&right_digit_len);
            }
            return left_digit.cmp(right_digit);
        } else {
            if left[i] != right[j] {
                return left[i].cmp(&right[j]);
            }
            i += 1;
            j += 1;
        }
    }
    left.len().cmp(&right.len())
}

/*
*opendir()* function:
opens a directory stream corresponding to
the directory name, and returns a pointer to the directory stream.
The stream is positioned at the first entry in the directory.
*/

struct OpenDir {
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
    fn new(path: *const c_char) -> Option<Self> {
        let dir = unsafe { opendir(path) };
        if dir.is_null() {
            return None;
        }
        let fd = unsafe { libc::dirfd(dir) };
        if fd.is_negative() {
            // explicitely closing dir
            // probably overkill, since main exits if new() returns none
            unsafe { libc::closedir(dir) };
            return None;
        }
        Some(Self { dir, fd })
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
struct DirEntry {
    dirfd: i32,
    dirent: *mut libc::dirent,
}

enum EntryType {
    Directory,
    RegularFile,
    SymLink,
    Unknown,
    Other,
}

impl EntryType {
    const fn emoji_view(&self) -> &str {
        match self {
            EntryType::Directory => " 🗂️  ",
            EntryType::RegularFile => " 📄 ",
            EntryType::SymLink => " 🔗 ",
            EntryType::Unknown => " ? ",
            EntryType::Other => " ",
        }
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
struct Metadata(libc::stat);
#[allow(unused)]
impl Metadata {
    fn new(entry: &DirEntry) -> Option<Self> {
        let mut stat_buf = core::mem::MaybeUninit::<libc::stat>::uninit();
        let name = unsafe { (*entry.dirent).d_name };
        let s = unsafe { fstatat(entry.dirfd, name.as_ptr(), stat_buf.as_mut_ptr(), 0) };
        if s == 0 {
            Some(Self(unsafe { stat_buf.assume_init() }))
        } else {
            None
        }
    }
    // to make it cross compile eg arm-unknown-linux-gnueabihf size is i32 not i64

    fn size(&self) -> usize {
        self.0.st_size as usize
    }
    fn n_link(&self) -> usize {
        self.0.st_nlink as usize
    }
    // need to format properly, not as timestamp
    // clippy!! where are you my Lord and Savior?!
    fn last_modified_fmt(&self) -> Option<[u8; 15]> {
        //example output 06/06/26 16:11 ie 14 bytes + null byte
        // need to consider this then pushing to buffer
        let mut char_buf: [u8; 15] = [0u8; 15];
        let mut tm = core::mem::MaybeUninit::<libc::tm>::zeroed();
        unsafe {
            let mtime = &self.0.st_mtime as *const libc::time_t;
            // https://linux.die.net/man/3/localtime_r
            // can return null
            let tm_ptr = libc::localtime_r(mtime, tm.as_mut_ptr());
            if tm_ptr.is_null() {
                return None;
            }
            let tm_ref = tm.assume_init_ref();
            //https://www.man7.org/linux/man-pages/man3/strftime.3.html
            // format
            let fmt = b"%d/%m/%y %H:%M\0";
            let writer = libc::strftime(
                char_buf.as_mut_ptr() as *mut libc::c_char,
                char_buf.len(),
                fmt.as_ptr() as *const libc::c_char,
                tm_ref,
            );
            // can be 0, so size is exceed the buffer size
            if writer == 0 {
                return None;
            }
        };
        Some(char_buf)
    }

    // https://www.man7.org/linux/man-pages/man3/getpwuid.3p.html
    fn user_bytes(&self) -> Option<&[u8]> {
        let pw = unsafe { libc::getpwuid(self.0.st_uid) };
        // can return null pointer
        if pw.is_null() {
            None
        } else {
            Some(unsafe { core::ffi::CStr::from_ptr((*pw).pw_name).to_bytes() })
        }
    }
    fn group_bytes(&self) -> Option<&[u8]> {
        let gr = unsafe { libc::getgrgid(self.0.st_gid) };
        // can return null pointer
        if gr.is_null() {
            None
        } else {
            Some(unsafe { core::ffi::CStr::from_ptr((*gr).gr_name).to_bytes() })
        }
    }

    // need to fetch all needed params
    // https://man7.org/linux/man-pages/man2/chmod.2.html
    // https://jameshfisher.com/2017/02/24/what-is-mode_t/
    fn mode_bytes(&self) -> [u8; 9] {
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

impl DirEntry {
    fn name(&self) -> &core::ffi::CStr {
        unsafe { core::ffi::CStr::from_ptr((*self.dirent).d_name.as_ptr()) }
    }
    fn entry_type(&self) -> EntryType {
        EntryType::from(unsafe { (*self.dirent).d_type })
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

impl core::fmt::Write for Buffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_bytes(s.as_bytes());
        Ok(())
    }
}

struct Buffer {
    buffer: [u8; BUFFER_SIZE],
    len: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            len: 0,
        }
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        let bytes_len = bytes.len();
        if self.len + bytes_len > self.buffer.len() {
            self.flush();
        }
        self.buffer[self.len..self.len + bytes_len].copy_from_slice(bytes);
        self.len += bytes_len;
    }

    fn flush(&mut self) {
        if self.len == 0 {
            return;
        }
        write_bytes(&self.buffer[0..self.len]);
        self.len = 0;
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

///// arean
struct Entry {
    name_offset: u32,  // 4 bytes ? maybe usize
    name_len: u8,      // 1 byte - since generally [u8;255]
    d_type: EntryType, // 1 byte
    metadata: Option<Metadata>,
}

struct EntryTable {
    arena: Vec<u8>,
    entries: Vec<Entry>,
    index: Vec<usize>,
}

impl EntryTable {
    fn new() -> Self {
        Self {
            arena: Vec::new(),
            entries: Vec::new(),
            index: Vec::new(),
        }
    }

    fn push_short(&mut self, entry: DirEntry) {
        let name = entry.name().to_bytes();
        let name_len = name.len();
        if name_len > 255 {
            return; // maybe truncate?
        }

        let offset = self.arena.len();
        self.arena.extend_from_slice(name);
        let idx = self.entries.len();
        self.entries.push(Entry {
            name_offset: offset as u32,
            name_len: name_len as u8,
            d_type: entry.entry_type(),
            metadata: None,
        });
        self.index.push(idx);
    }

    fn push_long(&mut self, entry: DirEntry) {
        let name = entry.name().to_bytes();
        let name_len = name.len();
        if name_len > 255 {
            return; // maybe truncate?
        }

        let offset = self.arena.len();
        self.arena.extend_from_slice(name);
        let idx = self.entries.len();
        // make it better, need error if none
        let metadata = Metadata::new(&entry);
        self.entries.push(Entry {
            name_offset: offset as u32,
            name_len: name_len as u8,
            d_type: entry.entry_type(),
            metadata,
        });
        self.index.push(idx);
    }

    fn name_by_index(&self, index: usize) -> &[u8] {
        let entry = &self.entries[index];
        let name_offset = entry.name_offset as usize;
        let name_len = entry.name_len;
        &self.arena[name_offset..name_offset + name_len as usize]
    }

    fn get_alignments(&self) -> Option<Alignments> {
        let mut max_n_link = 0;
        let mut max_size = 0;
        for i in 0..self.index.len() {
            if let Some(m) = &self.entries[i].metadata {
                max_n_link = max_n_link.max(digit_count(m.n_link()));
                max_size = max_size.max(digit_count(m.size()))
            } else {
                return None;
            }
        }
        Some(Alignments {
            n_link_width: max_n_link,
            size_width: max_size,
        })
    }

    fn sort_by_name(&mut self) {
        let arena = &self.arena;
        let entries = &self.entries;
        self.index.sort_unstable_by(|&a, &b| {
            let ea = &entries[a];
            let eb = &entries[b];
            let na =
                &arena[ea.name_offset as usize..ea.name_offset as usize + ea.name_len as usize];
            let nb =
                &arena[eb.name_offset as usize..eb.name_offset as usize + eb.name_len as usize];
            natural_cmp(na, nb)
        });
    }
}

enum Sort {
    Name,
    Size,
}

enum Display {
    Short,
    Long,
}

enum Mode {
    Stream,
    Alloc(Sort),
}

struct ReturnConfig {
    mode: Mode,
    display: Display,
    path: *const libc::c_char,
}

impl Default for ReturnConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Stream,
            display: Display::Short,
            path: c".".as_ptr(),
        }
    }
}

struct Alignments {
    n_link_width: usize,
    size_width: usize,
    // user
    // group
}

//probably replace aligmnets with struct {size,n_link,user,group} but for now size and n_link should be enough
struct Output {
    buffer: Buffer,
    alignments: Option<Alignments>,
}

impl Output {
    fn new(alignments: Option<Alignments>) -> Self {
        Self {
            buffer: Buffer::new(),
            alignments,
        }
    }

    fn output_name_and_type(&mut self, name: &[u8], f_type: &[u8]) {
        self.buffer.push_bytes(f_type);
        self.buffer.push_bytes(name);
        self.buffer.push_bytes(b"\n");
    }
    //alignments could be none! need to handle
    fn output_metadata_w_alignments(&mut self, m: &Metadata) {
        if let Some(aligments) = &self.alignments {
            self.buffer.push_bytes(&m.mode_bytes());
            self.buffer.push_bytes(b"  ");
            let mut nlink_buf: IntBytes = [b' '; 20];
            let aligned = align_int(&mut nlink_buf, m.n_link(), aligments.n_link_width);
            self.buffer.push_bytes(aligned);
            self.buffer.push_bytes(b"  ");
            if let Some(user) = m.user_bytes() {
                self.buffer.push_bytes(user);
                self.buffer.push_bytes(b"  ");
            }
            if let Some(group) = m.group_bytes() {
                self.buffer.push_bytes(group);
                self.buffer.push_bytes(b"  ");
            }
            let mut size_buf: IntBytes = [b' '; 20];
            let aligned = align_int(&mut size_buf, m.size(), aligments.size_width);
            self.buffer.push_bytes(aligned);
            self.buffer.push_bytes(b"  ");
            if let Some(lm_time) = m.last_modified_fmt() {
                self.buffer.push_bytes(&lm_time);
                self.buffer.push_bytes(b"  ");
            }
        }
    }

    fn stream_short(&mut self, entry: DirEntry) {
        self.output_name_and_type(
            entry.name().to_bytes(),
            entry.entry_type().emoji_view().as_bytes(),
        );
    }
    // add all
    fn stream_long(&mut self, entry: DirEntry) {
        if let Some(m) = Metadata::new(&entry) {
            self.output_metadata_w_alignments(&m);
        }
        self.stream_short(entry);
    }

    fn push_arena_short(&mut self, arena: EntryTable) {
        for i in 0..arena.index.len() {
            let entry = &arena.entries[arena.index[i]];
            let e_type_str = entry.d_type.emoji_view();
            let name = arena.name_by_index(arena.index[i]);
            self.output_name_and_type(name, e_type_str.as_bytes());
        }
    }

    //prints short if none, need to decided if return as it is or handle error?
    fn push_arena_long(&mut self, arena: EntryTable) {
        for i in 0..arena.index.len() {
            let entry = &arena.entries[arena.index[i]];
            if let Some(m) = &entry.metadata {
                self.output_metadata_w_alignments(m);
            }
            let e_type_str = entry.d_type.emoji_view();
            let name = arena.name_by_index(arena.index[i]);
            self.output_name_and_type(name, e_type_str.as_bytes());
        }
    }
    fn flush(&mut self) {
        self.buffer.flush();
    }
}

// so seems libc handles linker and there is no entry hassle
#[unsafe(no_mangle)]
fn main(argc: i32, argv: *const *mut libc::c_char) -> i32 {
    let mut config = ReturnConfig::default();
    let mut sort: Option<Sort> = None;

    loop {
        let opt = unsafe { libc::getopt(argc, argv, c"slS".as_ptr()) };
        if opt == -1 {
            break;
        }
        match opt as u8 {
            b's' => sort = Some(Sort::Name),
            b'S' => sort = Some(Sort::Size),
            b'l' => config.display = Display::Long,
            _ => {}
        }
    }

    if let Some(s) = sort {
        config.mode = Mode::Alloc(s)
    }
    let mut output = Output::new(None);

    match (config.mode, config.display) {
        (Mode::Stream, Display::Short) => {
            let Some(dir) = OpenDir::new(config.path) else {
                //silently return, need to add error handling
                return -1;
            };
            //https://www.man7.org/linux/man-pages/man3/readdir.3.html
            // here we need to be very careful , readdir() returns raw pointer to the next entry,
            // so after each iteration, we can consider it as invalid and should not use after
            for entry in dir {
                output.stream_short(entry);
            }
        }
        (Mode::Stream, Display::Long) => {
            let alignments = Alignments {
                n_link_width: MAX_INT_LEN,
                size_width: MAX_INT_LEN,
            };
            let Some(dir) = OpenDir::new(config.path) else {
                //silently return, need to add error handling
                return -1;
            };
            output.alignments = Some(alignments);
            for entry in dir {
                output.stream_long(entry);
            }
        }
        (Mode::Alloc(sort), Display::Short) => {
            let mut arena = EntryTable::new();
            let Some(dir) = OpenDir::new(config.path) else {
                //silently return, need to add error handling
                return -1;
            };
            for entry in dir {
                arena.push_short(entry);
            }
            match sort {
                Sort::Name => arena.sort_by_name(),
                Sort::Size => (),
            }
            output.push_arena_short(arena);
        }
        (Mode::Alloc(sort), Display::Long) => {
            let mut arena = EntryTable::new();
            let Some(dir) = OpenDir::new(config.path) else {
                //silently return, need to add error handling
                return -1;
            };
            for entry in dir {
                arena.push_long(entry);
            }
            match sort {
                Sort::Name => arena.sort_by_name(),
                Sort::Size => (),
            }
            output.alignments = arena.get_alignments();
            if output.alignments.is_some() {
                output.push_arena_long(arena);
            }
        }
    }
    output.flush();
    0
}
