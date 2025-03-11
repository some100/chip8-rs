use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmuError {
    #[error("SDL2 Error: {0}")]
    Sdl(String),
    #[error("Failed to build window {0}")]
    Window(#[from] sdl2::video::WindowBuildError),
    #[error("Integer overflow {0}")]
    Integer(#[from] sdl2::IntegerOrSdlError),
    #[error("Failed to convert usize to i32 {0}")]
    IntCast(#[from] std::num::TryFromIntError),
    #[error("Stream Error: {0}")]
    Stream(#[from] rodio::StreamError),
    #[error("Play Error: {0}")]
    Play(#[from] rodio::PlayError),
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Stack Error: {0}")]
    Stack(String),
    #[error("Program exited")]
    Exit(),
    #[error("Invalid instruction {0}")]
    Invalid(u16),
}
