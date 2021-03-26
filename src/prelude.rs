
//! Prelude - Include traits for hal

pub use crate::hal::digital::v2::*;
pub use crate::hal::prelude::*; // embedded hal traits // for some reason v2 is not exported in the ehal prelude

pub use crate::gpio::GpioExt as _stm32l4_hal_GpioExt;
pub use crate::rcc::RccExt as _stm32l4_hal_RccExt;