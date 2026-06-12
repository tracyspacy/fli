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
    MissingMetadata,
    AlignmentWidthError,
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
            FliError::MissingMetadata => 10,
            FliError::AlignmentWidthError => 11,
        }
    }
}

pub type FliResult<T> = Result<T, FliError>;
