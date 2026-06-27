use crate::dir::{DirEntry, EntryType, Metadata};
use crate::errors::{FliError::NameLen, FliResult};
use crate::output_config::{Alignments, Sort, Width};
use crate::utils::{digit_count, natural_cmp, sort_index_by};
use alloc::vec::Vec;
///// arean
pub struct Entry<M> {
    name_offset: usize,
    name_len: u8, // 1 byte - since generally [u8;255]
    sl_path_offset: usize,
    sl_path_len: u8,   // 1 byte - since generally [u8;255]
    d_type: EntryType, // 1 byte
    metadata: M,
}

pub struct EntryTable<M> {
    arena: Vec<u8>,
    entries: Vec<Entry<M>>,
    index: Vec<usize>,
}

pub type ShortTable = EntryTable<()>;
pub type LongTable = EntryTable<Metadata>;

impl EntryTable<()> {
    pub fn push(&mut self, entry: DirEntry) -> FliResult<()> {
        let name = entry.name().to_bytes();
        let name_len = name.len();
        if name_len > 255 {
            return Err(NameLen);
        }

        let offset = self.arena.len();
        self.arena.extend_from_slice(name);
        let idx = self.entries.len();
        self.entries.push(Entry {
            name_offset: offset,
            name_len: name_len as u8,
            sl_path_offset: 0, // not included
            sl_path_len: 0,    // not included
            d_type: entry.entry_type(),
            metadata: (),
        });
        self.index.push(idx);
        Ok(())
    }
    pub fn sort_by(&mut self) {
        self.sort_by_name();
    }
}

impl EntryTable<Metadata> {
    pub fn push(&mut self, entry: DirEntry) -> FliResult<()> {
        let entry_type = entry.entry_type();

        let name = entry.name();
        let name_bytes = name.to_bytes();
        let name_len = name_bytes.len();
        if name_len > 255 {
            return Err(NameLen);
        }
        let name_offset = self.arena.len();
        self.arena.extend_from_slice(name_bytes);

        let (sl_path_offset, sl_path_len) = if !entry_type.is_symlink() {
            (0, 0)
        } else {
            let (sl_path, sl_path_len) = entry.sym_link_with_value(name)?;
            let sl_path_offset = self.arena.len();
            self.arena.extend_from_slice(&sl_path[..sl_path_len]);
            (sl_path_offset, sl_path_len)
        };

        let idx = self.entries.len();

        let metadata = Metadata::new(&entry)?;
        self.entries.push(Entry {
            name_offset,
            name_len: name_len as u8,
            sl_path_offset,
            sl_path_len: sl_path_len as u8,
            d_type: entry.entry_type(),
            metadata,
        });
        self.index.push(idx);
        Ok(())
    }

    pub fn metadata_by_index(&self, index: usize) -> &Metadata {
        let entry = self.entry_by_index(index);
        &entry.metadata
    }

    pub fn get_alignments(&self) -> FliResult<Alignments> {
        let mut max_n_link = 0;
        let mut max_size = 0;
        for i in 0..self.index.len() {
            let m = &self.entries[i].metadata;
            max_n_link = max_n_link.max(digit_count(m.n_link()));
            max_size = max_size.max(digit_count(m.size()))
        }
        Ok(Alignments {
            n_link_width: Width::new(max_n_link)?,
            size_width: Width::new(max_size)?,
        })
    }
    fn sort_by_size(&mut self) {
        let entries = &self.entries;
        sort_index_by(&mut self.index, &|a: usize, b: usize| {
            let a_m = &entries[a].metadata;
            let b_m = &entries[b].metadata;
            a_m.size().cmp(&b_m.size())
        });
    }
    fn sort_by_time(&mut self) {
        let entries = &self.entries;
        sort_index_by(&mut self.index, &|a: usize, b: usize| {
            let a_m = &entries[a].metadata;
            let b_m = &entries[b].metadata;
            a_m.st_mtime().cmp(&b_m.st_mtime())
        });
    }

    pub fn sort_by(&mut self, sort_type: Sort) {
        match sort_type {
            Sort::Name => self.sort_by_name(),
            Sort::Size => self.sort_by_size(),
            Sort::Time => self.sort_by_time(),
        }
    }
}

impl<M> EntryTable<M> {
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            entries: Vec::new(),
            index: Vec::new(),
        }
    }

    pub fn indexes(&self) -> impl Iterator<Item = usize> {
        self.index.iter().copied()
    }

    pub fn entry_by_index(&self, index: usize) -> &Entry<M> {
        &self.entries[index]
    }

    pub fn entry_type_by_index(&self, index: usize) -> &EntryType {
        let entry = self.entry_by_index(index);
        &entry.d_type
    }

    pub fn name_by_index(&self, index: usize) -> &[u8] {
        let entry = &self.entries[index];
        let name_offset = entry.name_offset;
        let name_len = entry.name_len;
        &self.arena[name_offset..name_offset + name_len as usize]
    }
    pub fn sym_link_by_index(&self, index: usize) -> &[u8] {
        let entry = &self.entries[index];
        let symlink_offset = entry.sl_path_offset;
        let symlink_len = entry.sl_path_len as usize;
        &self.arena[symlink_offset..symlink_offset + symlink_len]
    }
    fn sort_by_name(&mut self) {
        let arena = &self.arena;
        let entries = &self.entries;
        sort_index_by(&mut self.index, &|a, b| {
            let ea: &Entry<M> = &entries[a];
            let eb: &Entry<M> = &entries[b];
            let na = &arena[ea.name_offset..ea.name_offset + ea.name_len as usize];
            let nb = &arena[eb.name_offset..eb.name_offset + eb.name_len as usize];
            natural_cmp(na, nb)
        });
    }
}
