use crate::errors::{FliError::AlignmentWidthError, FliResult};

pub const MAX_INT_LEN: usize = 20;
pub type IntBytes = [u8; MAX_INT_LEN];
pub const DEF_INT_BYTES: IntBytes = [b' '; MAX_INT_LEN];

pub enum Sort {
    Name,
    Size,
    Time,
}

pub enum Display {
    Short,
    Long,
}

pub enum Mode {
    Stream,
    Alloc(Sort),
}
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum View {
    Emoji = 0,
    Text = 5,
}

pub struct ReturnConfig {
    pub mode: Mode,
    pub display: Display,
    pub view: View,
    pub path: *const libc::c_char,
}

impl Default for ReturnConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Alloc(Sort::Name),
            display: Display::Short,
            view: View::Emoji,
            path: c".".as_ptr(),
        }
    }
}

pub struct Width(usize);
impl Width {
    pub fn new(val: usize) -> FliResult<Self> {
        if val <= MAX_INT_LEN {
            Ok(Self(val))
        } else {
            Err(AlignmentWidthError)
        }
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

//is it belong here?
pub struct Alignments {
    pub n_link_width: Width,
    pub size_width: Width,
    // user
    // group
}
