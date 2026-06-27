#[repr(u8)]
#[derive(Debug)]
pub enum FliError {
    OpenDirError,
    DirFd,
    FStatAt,
    NameLen,
    LocalTime,
    StrFTime,
    Getpwuid,
    Getgrgid,
    AlignmentWidthError,
    EntryAlreadyCachedErr,
    ReadLink,
    WrongEntryType,
}

impl FliError {
    pub fn to_exit_code(&self) -> i32 {
        match self {
            FliError::OpenDirError => 1,
            FliError::DirFd => 2,
            FliError::FStatAt => 3,
            FliError::NameLen => 4,
            FliError::LocalTime => 5,
            FliError::StrFTime => 6,
            FliError::Getpwuid => 7,
            FliError::Getgrgid => 8,
            FliError::AlignmentWidthError => 9,
            FliError::EntryAlreadyCachedErr => 10,
            FliError::ReadLink => 11,
            FliError::WrongEntryType => 12,
        }
    }
}

pub type FliResult<T> = Result<T, FliError>;
