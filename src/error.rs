#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SDL Error: {0}")]
    Sdl(#[from] sdl3::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
