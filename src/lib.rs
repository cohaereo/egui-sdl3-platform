pub mod conversions;
pub mod platform;

pub use conversions::*;
pub use platform::*;

pub type Result<T> = std::result::Result<T, sdl3::Error>;
