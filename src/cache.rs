/*
idea - tiny bytes cache to use for name/group to avoid syscall on each entry
premises:
1. there are should not be hundreds of unique names/groups in most cases
2. names/groups are repetative, so use traverse with promotion of most recently used  makes sense


*/

use crate::errors::{FliError::EntryAlreadyCachedErr, FliResult};

struct CacheEntry<const S: usize> {
    key: usize,
    value: [u8; S],
    size: usize,
}

impl<const S: usize> Default for CacheEntry<S> {
    fn default() -> Self {
        Self {
            key: 0,
            value: [0; S],
            size: 0,
        }
    }
}

impl<const S: usize> CacheEntry<S> {
    fn fill(&mut self, key: usize, value: &[u8]) {
        self.key = key;
        //truncating if len is > S
        let value_len = value.len().min(S);
        self.value[..value_len].copy_from_slice(&value[..value_len]);
        // self.size is always <= S
        self.size = value_len;
    }
    fn value_bytes(&self) -> &[u8] {
        &self.value[..self.size]
    }
}

pub struct ByteCache<const L: usize, const S: usize> {
    entries: [CacheEntry<S>; L],
    traverse: [usize; L],
    len: usize,
}

impl<const L: usize, const S: usize> ByteCache<L, S> {
    pub fn new() -> Self {
        Self {
            entries: core::array::from_fn(|_| CacheEntry::default()),
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
        let idx = self.traverse[self.len() - 1];
        self.entries[idx] = CacheEntry::default();
        self.len -= 1;
    }
    // not getting false positive on zeroed values since loop is based on len
    pub fn get(&mut self, key: usize) -> Option<&[u8]> {
        for i in 0..self.len {
            let idx = self.traverse[i];
            if key == self.entries[idx].key {
                self.promote(i);
                return Some(self.entries[idx].value_bytes());
            }
        }
        None
    }

    // checks if key is already cached, we shouldn't allow rewrites - uid/gid are stable
    fn key_exists(&self, key: usize) -> bool {
        for i in 0..self.len {
            let idx = self.traverse[i];
            if key == self.entries[idx].key {
                return true;
            }
        }
        false
    }

    pub fn insert(&mut self, key: usize, value: &[u8]) -> FliResult<()> {
        if self.key_exists(key) {
            return Err(EntryAlreadyCachedErr);
        }
        if self.is_full() {
            self.evict();
        }
        let len = self.len();
        let idx = self.traverse[len];
        self.entries[idx].fill(key, value);
        // last recently used, should be promoted
        self.promote(len);
        self.len += 1;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cache::ByteCache;
    use crate::errors::FliError;
    const K_V_INSERT_INITIAL: [(usize, &[u8]); 5] = [
        (1, b"one"),
        (2, b"two"),
        (3, b"three"),
        (4, b"four"),
        (5, b"five"),
    ];
    const K_V_EVICT_ON_INSERT: [(usize, &[u8]); 5] = [
        (6, b"six"),
        (2, b"two"),
        (3, b"three"),
        (4, b"four"),
        (5, b"five"),
    ];
    const TRAVERSE_ORDER_INIT: [usize; 5] = [0, 1, 2, 3, 4];
    const TRAVERSE_ORDER_INIT_INSERTS: [usize; 5] = [4, 3, 2, 1, 0];
    const TRAVERSE_ORDER_EVICT_ON_INSERT: [usize; 5] = [0, 4, 3, 2, 1];

    //insert 5 and promote after get(3) => TRAVERSE_ORDER_INIT_INSERTS promotes 2
    const TRAVERSE_ORDER_PROMOTE_ON_GET: [usize; 5] = [2, 4, 3, 1, 0];

    // scenariio to test logic
    #[test]
    fn test_insert_evict() {
        // entries: [CacheEntry[u8;32];5] - entry has a key and value
        let mut cache: ByteCache<5, 32> = ByteCache::new();
        // test initial traverse order
        assert_eq!(cache.traverse, TRAVERSE_ORDER_INIT);
        for (k, v) in K_V_INSERT_INITIAL {
            cache.insert(k, v).expect("insertion failed");
        }
        // test change of traverse order after insertions (each insert promotes)
        // entries keys    [1,2,3,4,5]
        // traverse        [4,3,2,1,0]
        // it means that in for ex. get(4) loop will make just 2 checks entries[4] which is 5 => next entries[3] which is 4
        assert_eq!(cache.traverse, TRAVERSE_ORDER_INIT_INSERTS);
        cache.insert(6, b"six").expect("insertion failed");
        // since cache is full, on insert it will evict Entry with last index in traverse => evicts entries[0]==1
        // and replaces with a new value so entries[0]=6
        // and since it inserts it promotes value => 0 becomes first element in traverse
        // entries keys     [6,2,3,4,5]
        // traverse         [0,4,3,2,1]
        assert_eq!(cache.traverse, TRAVERSE_ORDER_EVICT_ON_INSERT);
        for (i, &(k, v)) in K_V_EVICT_ON_INSERT.iter().enumerate() {
            assert_eq!(cache.entries[i].key, k);
            assert_eq!(cache.entries[i].value_bytes(), v); // rn values are with trailing zeroes. 
        }
    }

    // not getting false positive on zeroed values
    #[test]
    fn get_zero_id_test() {
        //empty cache
        let mut cache: ByteCache<5, 32> = ByteCache::new();
        cache.insert(1, b"one").expect("insertion failed");
        let res = cache.get(0);
        assert_eq!(res, None);
    }

    #[test]
    fn promote_on_get_test() {
        let mut cache: ByteCache<5, 32> = ByteCache::new();
        for (k, v) in K_V_INSERT_INITIAL {
            cache.insert(k, v).expect("insertion failed");
        }
        cache.get(3);
        assert_eq!(cache.traverse, TRAVERSE_ORDER_PROMOTE_ON_GET);
    }

    #[test]
    fn truncated_value_test() {
        let mut cache: ByteCache<5, 2> = ByteCache::new();
        cache.insert(1, b"one").expect("insertion failed");
        let v1 = cache.get(1);
        assert_eq!(v1, Some(b"on".as_slice()));
    }

    #[test]
    fn rewrite_existing_key_test() {
        let mut cache: ByteCache<5, 32> = ByteCache::new();
        cache.insert(1, b"one").expect("insertion failed");
        let attempt_to_rewrite = cache.insert(1, b"two");
        assert!(matches!(
            attempt_to_rewrite,
            Err(FliError::EntryAlreadyCachedErr)
        ));
    }
    // test ,that there isno trailing 0s
    #[test]
    fn get_len_test() {
        let mut cache: ByteCache<5, 32> = ByteCache::new();
        cache.insert(1, b"one").expect("insertion failed");
        let v1 = cache.get(1).unwrap();
        assert_eq!(v1.len(), b"one".len());
        assert_eq!(v1, b"one");
    }
}
