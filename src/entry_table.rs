use crate::dir::{DirEntry, EntryType, Metadata};
use crate::errors::{
    FliError::{MissingMetadata, NameLen},
    FliResult,
};
use crate::output_config::{Alignments, Width};
use crate::utils::{digit_count, natural_cmp};
use alloc::vec::Vec;
///// arean
pub struct Entry {
    name_offset: usize,
    name_len: u8, // 1 byte - since generally [u8;255]
    sl_path_offset: usize,
    sl_path_len: u8,       // 1 byte - since generally [u8;255]
    pub d_type: EntryType, // 1 byte
    pub metadata: Option<Metadata>,
}

pub struct EntryTable {
    arena: Vec<u8>,
    pub entries: Vec<Entry>,
    pub index: Vec<usize>,
}

impl EntryTable {
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            entries: Vec::new(),
            index: Vec::new(),
        }
    }

    pub fn push_short(&mut self, entry: DirEntry) {
        let name = entry.name().to_bytes();
        let name_len = name.len();
        if name_len > 255 {
            return; // maybe truncate?
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
            metadata: None,
        });
        self.index.push(idx);
    }

    pub fn push_long(&mut self, entry: DirEntry) -> FliResult<()> {
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
            metadata: Some(metadata),
        });
        self.index.push(idx);
        Ok(())
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

    pub fn get_alignments(&self) -> FliResult<Alignments> {
        let mut max_n_link = 0;
        let mut max_size = 0;
        for i in 0..self.index.len() {
            if let Some(m) = &self.entries[i].metadata {
                max_n_link = max_n_link.max(digit_count(m.n_link()));
                max_size = max_size.max(digit_count(m.size()))
            } else {
                return Err(MissingMetadata);
            }
        }
        Ok(Alignments {
            n_link_width: Width::new(max_n_link)?,
            size_width: Width::new(max_size)?,
        })
    }

    pub fn sort_by_name(&mut self) {
        let arena = &self.arena;
        let entries = &self.entries;
        self.index.sort_unstable_by(|&a, &b| {
            let ea = &entries[a];
            let eb = &entries[b];
            let na = &arena[ea.name_offset..ea.name_offset + ea.name_len as usize];
            let nb = &arena[eb.name_offset..eb.name_offset + eb.name_len as usize];
            natural_cmp(na, nb)
        });
    }
}
