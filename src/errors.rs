#[repr(u8)]
#[derive(Debug)]
pub enum FliError {
    OpenDirError,
    DirFd,
    FStatAt,
    NameLen,
    LocalTime,
    StrFTime,
    MissingAlignments,
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
            FliError::MissingAlignments => 7,
            FliError::Getpwuid => 8,
            FliError::Getgrgid => 9,
            FliError::AlignmentWidthError => 10,
            FliError::EntryAlreadyCachedErr => 11,
            FliError::ReadLink => 12,
            FliError::WrongEntryType => 13,
        }
    }
}

pub type FliResult<T> = Result<T, FliError>;
