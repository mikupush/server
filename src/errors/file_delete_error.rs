pub enum FileDeleteError {
    NotFound,
    IO { message: String },
    DB { message: String },
}

impl From<diesel::result::Error> for FileDeleteError {
    fn from(value: diesel::result::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<r2d2::Error> for FileDeleteError {
    fn from(value: r2d2::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<std::io::Error> for FileDeleteError {
    fn from(value: std::io::Error) -> Self {
        Self::IO { message: value.to_string() }
    }
}
