#[cfg(feature = "board-f411-nucleo")]
pub mod f411_nucleo;
#[cfg(feature = "board-f103c8-bluepill")]
pub mod f103c8_bluepill;

#[cfg(feature = "board-f411-nucleo")]
pub use f411_nucleo as current;
#[cfg(feature = "board-f103c8-bluepill")]
pub use f103c8_bluepill as current;

#[cfg(not(any(feature = "board-f411-nucleo", feature = "board-f103c8-bluepill")))]
compile_error!("no board feature enabled");

#[cfg(all(feature = "board-f411-nucleo", feature = "board-f103c8-bluepill"))]
compile_error!("multiple board features enabled");
