use crate::cache::ByteCache;
use crate::dir::{DirEntry, Metadata};
use crate::entry_table::{LongTable, ShortTable};
use crate::errors::FliResult;
use crate::output_config::{Alignments, DEF_INT_BYTES, View};
use crate::utils::align_int;
use libc::{STDOUT_FILENO, write};
//4 kb sounds nice
const BUFFER_SIZE: usize = 4096;
const CACHE_SIZE: usize = 50;
// should be enough
// seems 30-32 is general limit
const CACHE_VALUE_SIZE: usize = 32;
const DELIMITER: &[u8] = b"  ";
const NEW_LINE: &[u8] = b"\n";
const ARROW: &[u8] = b" -> ";

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
// no gain in size, but nicer
impl Drop for Buffer {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct OutputShort {
    buffer: Buffer,
}

impl OutputShort {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
        }
    }
    fn output_name_and_type(&mut self, name: &[u8], f_type: &[u8]) {
        self.buffer.push_bytes(f_type);
        self.buffer.push_bytes(name);
        self.buffer.push_bytes(NEW_LINE);
    }
    pub fn stream_short(&mut self, entry: DirEntry, view: View) {
        self.output_name_and_type(entry.name().to_bytes(), entry.entry_type().view_fmt(view));
    }
    pub fn push_arena_short(&mut self, arena: ShortTable, view: View) {
        arena.indexes().for_each(|idx| {
            let e_type_str = arena.entry_type_by_index(idx).view_fmt(view);
            let name = arena.name_by_index(idx);
            self.output_name_and_type(name, e_type_str);
        });
    }
}

//probably replace aligmnets with struct {size,n_link,user,group} but for now size and n_link should be enough
pub struct OutputLong {
    buffer: Buffer,
    names_cache: ByteCache<CACHE_SIZE, CACHE_VALUE_SIZE>,
    groups_cache: ByteCache<CACHE_SIZE, CACHE_VALUE_SIZE>,
    alignments: Alignments,
}

impl OutputLong {
    pub fn new(alignments: Alignments) -> Self {
        Self {
            buffer: Buffer::new(),
            names_cache: ByteCache::new(),
            groups_cache: ByteCache::new(),
            alignments,
        }
    }
    // single fn to reduce bin size
    fn output_name_type_link(&mut self, name: &[u8], f_type: &[u8], link: Option<&[u8]>) {
        self.buffer.push_bytes(f_type);
        self.buffer.push_bytes(name);
        if let Some(link) = link {
            self.buffer.push_bytes(ARROW);
            self.buffer.push_bytes(link);
        }
        self.buffer.push_bytes(NEW_LINE);
    }

    //alignments could be none! need to handle
    fn output_metadata_w_alignments(&mut self, m: &Metadata) -> FliResult<()> {
        let alignments = &self.alignments;
        self.buffer.push_bytes(&m.mode_bytes());
        self.buffer.push_bytes(DELIMITER);
        let mut nlink_buf = DEF_INT_BYTES;
        let aligned = align_int(&mut nlink_buf, m.n_link(), &alignments.n_link_width);
        self.buffer.push_bytes(aligned);
        self.buffer.push_bytes(DELIMITER);

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
        self.buffer.push_bytes(DELIMITER);

        let gid = m.get_gr_gid();
        let group_bytes = if let Some(cached_group) = self.groups_cache.get(gid) {
            cached_group
        } else {
            let group = m.group_bytes()?;
            self.groups_cache.insert(gid, group)?;
            group
        };
        self.buffer.push_bytes(group_bytes);
        self.buffer.push_bytes(DELIMITER);

        let mut size_buf = DEF_INT_BYTES;
        let aligned = align_int(&mut size_buf, m.size(), &alignments.size_width);
        self.buffer.push_bytes(aligned);
        self.buffer.push_bytes(DELIMITER);
        let lm_time = m.last_modified_fmt()?;
        self.buffer.push_bytes(&lm_time);
        self.buffer.push_bytes(DELIMITER);

        Ok(())
    }

    pub fn stream_long(&mut self, entry: DirEntry, view: View) -> FliResult<()> {
        let metadata = Metadata::new(&entry)?;
        self.output_metadata_w_alignments(&metadata)?;
        let entry_type = entry.entry_type();
        let f_type = entry_type.view_fmt(view);
        let name = entry.name();
        let (link, path_buf);
        if !entry_type.is_symlink() {
            link = None;
        } else {
            let (path, path_len) = entry.sym_link_with_value(name)?;
            path_buf = path;
            link = Some(&path_buf[..path_len]);
        }
        self.output_name_type_link(name.to_bytes(), f_type, link);
        Ok(())
    }

    pub fn push_arena_long(&mut self, arena: LongTable, view: View) -> FliResult<()> {
        for idx in arena.indexes() {
            let m = arena.metadata_by_index(idx);
            self.output_metadata_w_alignments(m)?;
            let entry_type = arena.entry_type_by_index(idx);
            let type_fmt = entry_type.view_fmt(view);
            let name = arena.name_by_index(idx);

            let link = if !entry_type.is_symlink() {
                None
            } else {
                Some(arena.sym_link_by_index(idx))
            };
            self.output_name_type_link(name, type_fmt, link);
        }
        Ok(())
    }
}
