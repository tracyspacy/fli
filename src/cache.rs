/*
idea - tiny bytes cache to use for name/group to avoid syscall on each entry
premises:
1. there are should not be hundreds of unique names/groups in most cases
2. names/groups are repetative, so use traverse with promotion of most recently used  makes sense


*/

/// TODO:
/// -keys make option, so 0 != not exist
/// -add tests https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=dfe8b36a80d31ad7325ad716a631c031
//

#[derive(Debug)]
pub struct ByteCache<const L: usize, const S: usize> {
    keys: [usize; L],
    values: [[u8; S]; L],
    traverse: [usize; L],
    len: usize,
}

impl<const L: usize, const S: usize> ByteCache<L, S> {
    pub fn new() -> Self {
        Self {
            keys: [0usize; L],
            values: [[0u8; S]; L],
            traverse: core::array::from_fn(|i| i),
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_full(&self) -> bool {
        self.len() == L
    }

    // moving most recently used to front ie traverse[0]
    // shifting other so [a,b,c,d,e,f] -> [d,a,b,c,e,f]
    fn promote(&mut self, position: usize) {
        self.traverse[0..=position].rotate_right(1);
    }

    fn evict(&mut self) {
        if self.len() == 0 {
            return;
        }
        let last = self.len() - 1;
        let idx = self.traverse[last];
        self.keys[idx] = 0usize;
        self.values[idx] = [0u8; S];
        self.len -= 1;
    }

    pub fn get(&mut self, key: usize) -> Option<&[u8]> {
        for i in 0..self.len {
            let idx = self.traverse[i];
            if key == self.keys[idx] {
                self.promote(i);
                return Some(&self.values[idx]);
            }
        }
        None
    }

    pub fn insert(&mut self, key: usize, value: &[u8]) {
        if self.is_full() {
            self.evict();
        }
        let len = self.len();
        let idx = self.traverse[len];
        self.keys[idx] = key;
        //truncating if len is > S
        let value_len = value.len().min(S);
        self.values[idx][..value_len].copy_from_slice(&value[..value_len]);
        // last recently used, should be promoted
        self.promote(len);
        self.len += 1;
    }
}
