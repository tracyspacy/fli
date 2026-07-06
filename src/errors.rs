#[repr(u8)]
#[derive(Debug)]
pub enum FliError {
    OpenDirError = 1,
    DirFd = 2,
    FStatAt = 3,
    NameLen = 4,
    LocalTime = 5,
    StrFTime = 6,
    Getpwuid = 7,
    Getgrgid = 8,
    AlignmentWidthError = 9,
    EntryAlreadyCachedErr = 10,
    ReadLink = 11,
    WrongEntryType = 12,
    FindEntry = 13,
    NoSuchFileOrDir = 14,
}

pub type FliResult<T> = Result<T, FliError>;
