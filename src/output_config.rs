pub enum Sort {
    Name,
    Size,
}

pub enum Display {
    Short,
    Long,
}

pub enum Mode {
    Stream,
    Alloc(Sort),
}

pub struct ReturnConfig {
    pub mode: Mode,
    pub display: Display,
    pub path: *const libc::c_char,
}

impl Default for ReturnConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Stream,
            display: Display::Short,
            path: c".".as_ptr(),
        }
    }
}

//is it belong here?
pub struct Alignments {
    pub n_link_width: usize,
    pub size_width: usize,
    // user
    // group
}
