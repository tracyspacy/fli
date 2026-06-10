use crate::dir::{DirEntry, EntryType, Metadata};
use crate::errors::{FliError, FliResult};
use crate::output_config::{Alignments, Width};
use crate::utils::{digit_count, natural_cmp};
use alloc::vec::Vec;
///// arean
pub struct Entry {
    name_offset: u32,      // 4 bytes ? maybe usize
    name_len: u8,          // 1 byte - since generally [u8;255]
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
            name_offset: offset as u32,
            name_len: name_len as u8,
            d_type: entry.entry_type(),
            metadata: None,
        });
        self.index.push(idx);
    }

    pub fn push_long(&mut self, entry: DirEntry) -> FliResult<()> {
        let name = entry.name().to_bytes();
        let name_len = name.len();
        if name_len > 255 {
            return Err(crate::errors::FliError::NameLenError); // maybe truncate?
        }
        let offset = self.arena.len();
        self.arena.extend_from_slice(name);
        let idx = self.entries.len();
        // make it better, need error if none
        let metadata = Metadata::new(&entry)?;
        self.entries.push(Entry {
            name_offset: offset as u32,
            name_len: name_len as u8,
            d_type: entry.entry_type(),
            metadata: Some(metadata),
        });
        self.index.push(idx);
        Ok(())
    }

    pub fn name_by_index(&self, index: usize) -> &[u8] {
        let entry = &self.entries[index];
        let name_offset = entry.name_offset as usize;
        let name_len = entry.name_len;
        &self.arena[name_offset..name_offset + name_len as usize]
    }

    pub fn get_alignments(&self) -> FliResult<Alignments> {
        let mut max_n_link = 0;
        let mut max_size = 0;
        for i in 0..self.index.len() {
            if let Some(m) = &self.entries[i].metadata {
                max_n_link = max_n_link.max(digit_count(m.n_link()));
                max_size = max_size.max(digit_count(m.size()))
            } else {
                return Err(FliError::MissingMetadataError);
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
            let na =
                &arena[ea.name_offset as usize..ea.name_offset as usize + ea.name_len as usize];
            let nb =
                &arena[eb.name_offset as usize..eb.name_offset as usize + eb.name_len as usize];
            natural_cmp(na, nb)
        });
    }
}
