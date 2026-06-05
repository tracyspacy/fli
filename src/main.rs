#![no_std]
#![no_main]

use core::ffi::c_char;

use libc::{STDOUT_FILENO, opendir, write};
extern crate alloc;
use alloc::vec::Vec;

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

impl OpenDir {
    // how correct it is , if default is just same folder
    fn new(path: *const c_char) -> Self {
        //opendir can be null!!!! add check
        let dir = unsafe { opendir(path) };
        let fd = unsafe { libc::dirfd(dir) }; // can return error!

        Self { dir, fd }
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
            EntryType::Directory => " 🗂️ ",
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

    fn push(&mut self, entry: &DirEntry) {
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
        });
        self.index.push(idx);
    }

    fn name_by_index(&self, index: usize) -> &[u8] {
        let entry = &self.entries[index];
        let name_offset = entry.name_offset as usize;
        let name_len = entry.name_len;
        &self.arena[name_offset..name_offset + name_len as usize]
    }

    fn print(&self, buffer: &mut Buffer) {
        for i in 0..self.index.len() {
            let entry = &self.entries[self.index[i]];
            let e_type_str = entry.d_type.emoji_view();
            let name = self.name_by_index(self.index[i]);
            buffer.push_bytes(e_type_str.as_bytes());
            buffer.push_bytes(name);
            buffer.push_bytes(b"\n");
        }
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

// so seems libc handles linker and there is no entry hassle
#[unsafe(no_mangle)]
fn main(argc: i32, argv: *const *mut libc::c_char) {
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

    let mut buffer = Buffer::new();
    match config.mode {
        Mode::Stream => {
            let dir = OpenDir::new(config.path);
            //https://www.man7.org/linux/man-pages/man3/readdir.3.html
            // here we need to be very careful , readdir() returns raw pointer to the next entry,
            // so after each iteration, we can consider it as invalid and should not use after
            for entry in dir {
                buffer.push_bytes(entry.entry_type().emoji_view().as_bytes());
                buffer.push_bytes(entry.name().to_bytes());
                buffer.push_bytes("\n".as_bytes());
            }
        }
        Mode::Alloc(sort_opt) => {
            let dir = OpenDir::new(config.path);
            let mut arena = EntryTable::new();
            for entry in dir {
                arena.push(&entry);
            }
            match sort_opt {
                Sort::Name => arena.sort_by_name(),
                Sort::Size => write_bytes("sort_size".as_bytes()),
            }
            arena.print(&mut buffer);
        }
    }
    buffer.flush();
}
