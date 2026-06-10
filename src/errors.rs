#[repr(u8)]
pub enum FliError {
    OpenDirError,
    DirFdError,
    FStatAtError,
    NameLenError,
    LocalTimeError,
    StrFTimeError,
    MissingAlignmentsError,
    GetpwuidError,
    GetgrgidError,
    MissingMetadataError,
    AlignmentWidthError,
}

impl FliError {
    pub fn to_exit_code(&self) -> i32 {
        match self {
            FliError::OpenDirError => 1,
            FliError::DirFdError => 2,
            FliError::FStatAtError => 3,
            FliError::NameLenError => 4,
            FliError::LocalTimeError => 5,
            FliError::StrFTimeError => 6,
            FliError::MissingAlignmentsError => 7,
            FliError::GetpwuidError => 8,
            FliError::GetgrgidError => 9,
            FliError::MissingMetadataError => 10,
            FliError::AlignmentWidthError => 11,
        }
    }
}

pub type FliResult<T> = Result<T, FliError>;
