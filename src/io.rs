use crate::dir::{DirEntry, Metadata};
use crate::entry_table::EntryTable;
use crate::output_config::Alignments;
use crate::utils::{IntBytes, align_int};
use libc::{STDOUT_FILENO, write};
//4 kb sounds nice
const BUFFER_SIZE: usize = 4096;

// helper write
fn write_bytes(bytes: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, bytes.as_ptr() as *const _, bytes.len());
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

//probably replace aligmnets with struct {size,n_link,user,group} but for now size and n_link should be enough
pub struct Output {
    buffer: Buffer,
    pub alignments: Option<Alignments>,
}

impl Output {
    pub fn new(alignments: Option<Alignments>) -> Self {
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

    pub fn stream_short(&mut self, entry: DirEntry) {
        self.output_name_and_type(
            entry.name().to_bytes(),
            entry.entry_type().emoji_view().as_bytes(),
        );
    }
    // add all
    pub fn stream_long(&mut self, entry: DirEntry) {
        if let Some(m) = Metadata::new(&entry) {
            self.output_metadata_w_alignments(&m);
        }
        self.stream_short(entry);
    }

    pub fn push_arena_short(&mut self, arena: EntryTable) {
        for i in 0..arena.index.len() {
            let entry = &arena.entries[arena.index[i]];
            let e_type_str = entry.d_type.emoji_view();
            let name = arena.name_by_index(arena.index[i]);
            self.output_name_and_type(name, e_type_str.as_bytes());
        }
    }

    //prints short if none, need to decided if return as it is or handle error?
    pub fn push_arena_long(&mut self, arena: EntryTable) {
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
    pub fn flush(&mut self) {
        self.buffer.flush();
    }
}
