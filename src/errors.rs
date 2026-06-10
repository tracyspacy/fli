pub enum FliError {
    OpenDirError,
    DirFdError,
}

impl FliError {
    pub fn to_exit_code(&self) -> i32 {
        match self {
            FliError::OpenDirError => 1,
            FliError::DirFdError => 2,
        }
    }
}

pub type FliResult<T> = Result<T, FliError>;
