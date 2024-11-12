use libloading::Error as LibloadingError;

#[derive(Debug)]
pub enum Error {
    LibraryLoading(LibloadingError)
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryLoading(e) => write!(f, "Library loading error: {:?}", e)
        }
    }
}

impl std::error::Error for Error {

}
