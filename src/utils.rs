use crate::output_config::{IntBytes, Width};

// helper - primitive dgit counter
//helper
#[inline]
pub fn digit_count(mut n: usize) -> usize {
    if n < 10 {
        return 1;
    }
    if n < 100 {
        return 2;
    }
    if n < 1_000 {
        return 3;
    }
    if n < 10_000 {
        return 4;
    }
    if n < 100_000 {
        return 5;
    }
    if n < 1_000_000 {
        return 6;
    }
    if n < 10_000_000 {
        return 7;
    }
    if n < 100_000_000 {
        return 8;
    }
    // should be less than u32:max len
    if n < 1_000_000_000 {
        return 9;
    }

    let mut c = 9;
    while n >= 1_000_000_000 {
        n /= 10;
        c += 1;
    }
    c
}

// helper to align int in bytes array if max len 5 but int is 12 ie 2 [b' ',b' ',b' ',b'1',b'2']

pub fn align_int<'a>(buf: &'a mut IntBytes, digit: usize, width: &Width) -> &'a [u8] {
    let width = width.get();
    buf.fill(b' ');
    if digit == 0 {
        buf[width - 1] = b'0';
        return &buf[..width];
    }
    let mut tmp = digit;
    let mut pos = width;
    while tmp > 0 {
        pos -= 1;
        // b'0' = [48] + res of % till 9=> ascii numbers 0-9
        buf[pos] = b'0' + (tmp % 10) as u8;
        tmp /= 10;
    }
    buf[..pos].fill(b' ');
    &buf[..width]
}

// helper for natural cmp
// TODO:
// need to consider register! now Videos goes before fli
//
pub fn natural_cmp(left: &[u8], right: &[u8]) -> core::cmp::Ordering {
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

// helper to reuse sort_unstable_by() with own cmp fn
// &dyn allows to optimize bin size for arm-unknown-linux-gnueabihf
// so it not duplicating std core::slice::sort::unstable::quicksort::quicksort
pub fn sort_index_by(index_vec: &mut [usize], cmp: &dyn Fn(usize, usize) -> core::cmp::Ordering) {
    index_vec.sort_unstable_by(|&a, &b| cmp(a, b))
}

#[cfg(test)]
mod test {
    use crate::output_config::DEF_INT_BYTES;

    use super::*;
    const DIGIT_AND_LEN: [(usize, usize); 7] = [
        (0, 1),
        (9, 1),
        (99, 2),
        (100, 3),
        (1234, 4),
        (200_000_000, 9),
        (2_000_000_000, 10),
    ];

    const INT_WIDTH_RES: [(usize, usize, &[u8]); 4] = [
        (0, 5, b"    0"),
        (1, 5, b"    1"),
        (12345, 5, b"12345"),
        (1, 20, b"                   1"),
    ];

    const NAMES_CMP: [(&[u8], &[u8], core::cmp::Ordering); 6] = [
        (b"test", b"test", core::cmp::Ordering::Equal),
        (b"test1", b"test2", core::cmp::Ordering::Less),
        (b"test100", b"test9", core::cmp::Ordering::Greater),
        (b"a", b"aa", core::cmp::Ordering::Less),
        (b"aaaa", b"bbbb", core::cmp::Ordering::Less),
        (b"abcd", b"efg", core::cmp::Ordering::Less),
        //(b"Video", b"fli", core::cmp::Ordering::Greater), - known issue!
    ];

    #[test]
    fn digit_count_test() {
        for (digit, len) in DIGIT_AND_LEN {
            assert_eq!(digit_count(digit), len, "wrong digit count {}", digit);
        }
    }
    #[test]
    fn align_int_test() {
        for (digit, width, right) in INT_WIDTH_RES {
            let mut buf = DEF_INT_BYTES;
            let w = Width::new(width).unwrap();
            let left = align_int(&mut buf, digit, &w);
            assert_eq!(left, right, "wrong aligned {}", digit);
        }
    }

    #[test]
    fn natural_cmp_test() {
        for (name_l, name_r, res) in NAMES_CMP {
            let left = natural_cmp(name_l, name_r);
            assert_eq!(left, res, "wrong cmp {:?}", str::from_utf8(name_l),)
        }
    }
}
