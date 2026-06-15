/*
idea - tiny bytes cache to use for name/group to avoid syscall on each entry
premises:
1. there are should not be hundreds of unique names/groups in most cases
2. names/groups are repetative, so use traverse with promotion of most recently used  makes sense


*/

/// TODO:
/// -keys make option, so 0 != not exist
/// -add tests https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=dfe8b36a80d31ad7325ad716a631c031
//  -get() returns &[u8;l], not actual size, may have trailing zeroes

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

#[cfg(test)]
mod test {
    use crate::cache::ByteCache;
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

    // scenariio to test logic
    #[test]
    fn test_insert_evict() {
        //keys: [usize;5] values: [[u8;32];5]
        let mut cache: ByteCache<5, 32> = ByteCache::new();
        // test initial traverse order
        assert_eq!(cache.traverse, TRAVERSE_ORDER_INIT);
        for (k, v) in K_V_INSERT_INITIAL {
            cache.insert(k, v);
        }
        // test change of traverse order after insertions (each insert promotes)
        // keys     [1,2,3,4,5]
        // traverse [4,3,2,1,0]
        // it means that in for ex. get(4) loop will make just 2 checks kyes[4] which is 5 => next keys[3] which is 4
        assert_eq!(cache.traverse, TRAVERSE_ORDER_INIT_INSERTS);
        cache.insert(6, b"six");
        // since cache is full, on insert it will evict key/value with last index in traverse => evicts keys[0]==1
        // and replaces with a new value so key[0]=6 (same for values)
        // and since it inserts it promotes value => 0 becomes first element in traverse
        // keys     [6,2,3,4,5]
        // traverse [0,4,3,2,1]
        assert_eq!(cache.traverse, TRAVERSE_ORDER_EVICT_ON_INSERT);
        for (i, &(k, v)) in K_V_EVICT_ON_INSERT.iter().enumerate() {
            assert_eq!(cache.keys[i], k);
            assert_eq!(&cache.values[i][..v.len()], v); // rn values are with trailing zeroes. 
        }
    }
}
