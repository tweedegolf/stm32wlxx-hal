//! # HAL for the STM32WL family of microcontrollers
//!

#![no_std]
#![deny(broken_intra_doc_links)]

// If no target specified, print error message.
#[cfg(not(any(
    feature = "stm32wle5",
)))]
compile_error!("Target not found. A `--features <target-name>` is required.");

// TODO If any two or more targets are specified, print error message.


#[cfg(feature = "device-selected")]
use embedded_hal as hal ;

#[cfg(feature = "stm32wle5")]
pub use stm32wl::stm32wle5 as pac;


#[cfg(feature = "device-selected")]
#[deprecated(since = "0.6.0", note = "please use `pac` instead")]
#[doc(hidden)]
pub use crate::pac as device;

#[cfg(feature = "device-selected")]
#[deprecated(since = "0.6.0", note = "please use `pac` instead")]
#[doc(hidden)]
pub use crate::pac as stm32;

#[cfg(feature = "device-selected")]
pub mod gpio;