use crate::cache::ByteCache;
use crate::dir::{DirEntry, Metadata};
use crate::entry_table::EntryTable;
use crate::errors::{FliError::MissingAlignments, FliResult};
use crate::output_config::{Alignments, DEF_INT_BYTES};
use crate::utils::align_int;
use libc::{STDOUT_FILENO, write};
//4 kb sounds nice
const BUFFER_SIZE: usize = 4096;
const CACHE_SIZE: usize = 50;
// should be enough
// seems 30-32 is general limit
const CACHE_VALUE_SIZE: usize = 32;

// helper write
fn write_bytes(bytes: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, bytes.as_ptr() as *const _, bytes.len());
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
    names_cache: ByteCache<CACHE_SIZE, CACHE_VALUE_SIZE>,
    groups_cache: ByteCache<CACHE_SIZE, CACHE_VALUE_SIZE>,
    pub alignments: Option<Alignments>,
}

impl Output {
    pub fn new(alignments: Option<Alignments>) -> Self {
        Self {
            buffer: Buffer::new(),
            names_cache: ByteCache::new(),
            groups_cache: ByteCache::new(),
            alignments,
        }
    }

    fn output_name_and_type(&mut self, name: &[u8], f_type: &[u8]) {
        self.buffer.push_bytes(f_type);
        self.buffer.push_bytes(name);
        self.buffer.push_bytes(b"\n");
    }

    fn output_name_type_and_link(&mut self, name: &[u8], f_type: &[u8], link: &[u8]) {
        self.buffer.push_bytes(f_type);
        self.buffer.push_bytes(name);
        self.buffer.push_bytes(b" -> ");
        self.buffer.push_bytes(link);
        self.buffer.push_bytes(b"\n");
    }

    //alignments could be none! need to handle
    fn output_metadata_w_alignments(&mut self, m: &Metadata) -> FliResult<()> {
        if let Some(aligments) = &self.alignments {
            self.buffer.push_bytes(&m.mode_bytes());
            self.buffer.push_bytes(b"  ");
            let mut nlink_buf = DEF_INT_BYTES;
            let aligned = align_int(&mut nlink_buf, m.n_link(), &aligments.n_link_width);
            self.buffer.push_bytes(aligned);
            self.buffer.push_bytes(b"  ");

            //adding cache
            let uid = m.get_pw_uid();
            let name_bytes = if let Some(cached_name) = self.names_cache.get(uid) {
                cached_name
            } else {
                let user = m.user_bytes()?;
                self.names_cache.insert(uid, user)?;
                user
            };
            self.buffer.push_bytes(name_bytes);
            self.buffer.push_bytes(b"  ");

            let gid = m.get_gr_gid();
            let group_bytes = if let Some(cached_group) = self.groups_cache.get(gid) {
                cached_group
            } else {
                let group = m.group_bytes()?;
                self.groups_cache.insert(gid, group)?;
                group
            };
            self.buffer.push_bytes(group_bytes);
            self.buffer.push_bytes(b"  ");

            let mut size_buf = DEF_INT_BYTES;
            let aligned = align_int(&mut size_buf, m.size(), &aligments.size_width);
            self.buffer.push_bytes(aligned);
            self.buffer.push_bytes(b"  ");
            let lm_time = m.last_modified_fmt()?;
            self.buffer.push_bytes(&lm_time);
            self.buffer.push_bytes(b"  ");
        } else {
            return Err(MissingAlignments);
        }
        Ok(())
    }

    pub fn stream_short(&mut self, entry: DirEntry) {
        self.output_name_and_type(
            entry.name().to_bytes(),
            entry.entry_type().emoji_view().as_bytes(),
        );
    }

    pub fn stream_long(&mut self, entry: DirEntry) -> FliResult<()> {
        let metadata = Metadata::new(&entry)?;
        self.output_metadata_w_alignments(&metadata)?;
        let entry_type = entry.entry_type();
        let f_type = entry_type.emoji_view().as_bytes();
        let name = entry.name();
        if !entry_type.is_symlink() {
            self.output_name_and_type(name.to_bytes(), f_type);
        } else {
            let (path, path_len) = entry.sym_link_with_value(name)?;
            self.output_name_type_and_link(name.to_bytes(), f_type, &path[..path_len]);
        }
        Ok(())
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
    pub fn push_arena_long(&mut self, arena: EntryTable) -> FliResult<()> {
        for i in 0..arena.index.len() {
            let entry = &arena.entries[arena.index[i]];
            if let Some(m) = &entry.metadata {
                self.output_metadata_w_alignments(m)?;
            }
            let entry_type = &entry.d_type;
            let name = arena.name_by_index(arena.index[i]);
            if !entry_type.is_symlink() {
                self.output_name_and_type(name, entry_type.emoji_view().as_bytes());
            } else {
                let symlink = arena.sym_link_by_index(arena.index[i]);
                self.output_name_type_and_link(name, entry_type.emoji_view().as_bytes(), symlink);
            }
        }
        Ok(())
    }
    pub fn flush(&mut self) {
        self.buffer.flush();
    }
}
