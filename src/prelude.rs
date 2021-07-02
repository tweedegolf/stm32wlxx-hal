//! Prelude - Include traits for hal

pub use crate::hal::digital::v2::*;
pub use crate::hal::prelude::*; // embedded hal traits // for some reason v2 is not exported in the ehal prelude

pub use crate::flash::FlashExt as _stm32wl_hal_FlashExt;
pub use crate::gpio::GpioExt as _stm32wl_hal_GpioExt;
pub use crate::rcc::RccExt as _stm32wl_hal_RccExt;
pub use crate::time::U32Ext as _stm32wl_hal_U32Ext;
