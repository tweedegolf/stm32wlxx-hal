//! Serial Peripheral Interface (SPI) bus

use core::ptr;
use crate::hal::spi::{FullDuplex, Mode, Phase, Polarity};

use crate::gpio::{Alternate, Floating, Input, PA4, PA5, PA6, PA7, AF13};
use crate::rcc::{Clocks, APB3};
use crate::time::Hertz;

/// SPI error
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
}

#[doc(hidden)]
mod private {
    pub trait Sealed {}
}

/// SCK pin. This trait is sealed and cannot be implemented.
pub trait SckPin<SPI>: private::Sealed {}
/// MISO pin. This trait is sealed and cannot be implemented.
pub trait MisoPin<SPI>: private::Sealed {}
/// MOSI pin. This trait is sealed and cannot be implemented.
pub trait MosiPin<SPI>: private::Sealed {}

macro_rules! pins {
    ($spi:ident,
     $af:ident,
     SCK: [$($sck:ident),*],
     MISO: [$($miso:ident),*],
     MOSI: [$($mosi:ident),*],
     NSSOUT: [$($nssout:ident),*]) => {
        $(
            impl private::Sealed for $sck<Alternate<$af, Input<Floating>>> {}
            impl SckPin<$spi> for $sck<Alternate<$af, Input<Floating>>> {}
        )*
        $(
            impl private::Sealed for $miso<Alternate<$af, Input<Floating>>> {}
            impl MisoPin<$spi> for $miso<Alternate<$af, Input<Floating>>> {}
        )*
        $(
            impl private::Sealed for $mosi<Alternate<$af, Input<Floating>>> {}
            impl MosiPin<$spi> for $mosi<Alternate<$af, Input<Floating>>> {}
        )*
        $(
            impl private::Sealed for $nssout<Alternate<$af, Input<Floating>>> {}
            impl MosiPin<$spi> for $nssout<Alternate<$af, Input<Floating>>> {}
        )*
    }
}

/// SPI peripheral operating in full duplex master mode
/// This code has not been tested, please use with care
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

macro_rules! hal {
    ($($SPIX:ident: ($spiX:ident, $APBX:ident, $spiXen:ident, $spiXrst:ident, $pclkX:ident),)+) => {
        $(
            impl<SCK, MISO, MOSI> Spi<$SPIX, (SCK, MISO, MOSI)> {
                /// Configures the SPI peripheral to operate in full duplex master mode
                pub fn $spiX<F>(
                    spi: $SPIX,
                    pins: (SCK, MISO, MOSI),
                    mode: Mode,
                    freq: F,
                    clocks: Clocks,
                    apb3: &mut $APBX,
                ) -> Self
                where
                    F: Into<Hertz>,
                    SCK: SckPin<$SPIX>,
                    MISO: MisoPin<$SPIX>,
                    MOSI: MosiPin<$SPIX>,
                {
                    // enable or reset $SPIX
                    apb3.enr().modify(|_, w| w.$spiXen().set_bit());
                    apb3.rstr().modify(|_, w| w.$spiXrst().set_bit());
                    apb3.rstr().modify(|_, w| w.$spiXrst().clear_bit());

                    // FRXTH: RXNE event is generated if the FIFO level is greater than or equal to
                    //        8-bit
                    // DS: 8-bit data size
                    // SSOE: Slave Select output disabled
                    spi.cr2
                        .write(|w| unsafe {
                            w.frxth().set_bit().ds().bits(0b111).ssoe().clear_bit()
                        });

                    let br = Self::compute_baud_rate(clocks.$pclkX(), freq.into());

                    // CPHA: phase
                    // CPOL: polarity
                    // MSTR: master mode
                    // BR: 1 MHz
                    // SPE: SPI disabled
                    // LSBFIRST: MSB first
                    // SSM: enable software slave management (NSS pin free for other uses)
                    // SSI: set nss high = master mode
                    // CRCEN: hardware CRC calculation disabled
                    // BIDIMODE: 2 line unidirectional (full duplex)
                    spi.cr1.write(|w| unsafe {
                        w.cpha()
                            .bit(mode.phase == Phase::CaptureOnSecondTransition)
                            .cpol()
                            .bit(mode.polarity == Polarity::IdleHigh)
                            .mstr()
                            .set_bit()
                            .br()
                            .bits(br)
                            .spe()
                            .set_bit()
                            .lsbfirst()
                            .clear_bit()
                            .ssi()
                            .set_bit()
                            .ssm()
                            .set_bit()
                            .crcen()
                            .clear_bit()
                            .bidimode()
                            .clear_bit()
                    });

                    Spi { spi, pins }
                }

                /// Change the baud rate of the SPI
                pub fn reclock<F>(&mut self, freq: F, clocks: Clocks)
                    where F: Into<Hertz>
                {
                    self.spi.cr1.modify(|_, w| w.spe().clear_bit());
                    self.spi.cr1.modify(|_, w| {
                        unsafe {w.br().bits(Self::compute_baud_rate(clocks.$pclkX(), freq.into()));}
                        w.spe().set_bit()
                    });
                }

                fn compute_baud_rate(clocks: Hertz, freq: Hertz) -> u8 {
                    match clocks.0 / freq.0 {
                        0 => unreachable!(),
                        1..=2 => 0b000,
                        3..=5 => 0b001,
                        6..=11 => 0b010,
                        12..=23 => 0b011,
                        24..=39 => 0b100,
                        40..=95 => 0b101,
                        96..=191 => 0b110,
                        _ => 0b111,
                    }
                }

                /// Releases the SPI peripheral and associated pins
                pub fn free(self) -> ($SPIX, (SCK, MISO, MOSI)) {
                    (self.spi, self.pins)
                }
            }

            impl<PINS> FullDuplex<u8> for Spi<$SPIX, PINS> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
                        // reading a half-word)
                        return Ok(unsafe {
                            ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }

                fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.txe().bit_is_set() {
                        // NOTE(write_volatile) see note above
                        unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
                        return Ok(());
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl<PINS> crate::hal::blocking::spi::transfer::Default<u8> for Spi<$SPIX, PINS> {}

            impl<PINS> crate::hal::blocking::spi::write::Default<u8> for Spi<$SPIX, PINS> {}
        )+
    }
}

//TODO: Confirm that SUBGHZSPI is actually SPI3
use crate::pac::SPI3 as SUBGHZSPI;

hal! {
    SUBGHZSPI: (subghzspi, APB3, subghzspien, subghzspirst, pclk1),
}

/*
DEBUG_SUBGHZSPI_NSSOUT -> PA4
DEBUG_SUBGHZSPI_SCKOUT -> PA5
DEBUG_SUBGHZSPI_MISOOUT -> PA6
DEBUG_SUBGHZSPI_MOSIOUT -> PA7
 */

pins!(SUBGHZSPI, AF13,
      SCK: [PA5],
      MISO: [PA6],
      MOSI: [PA7],
      NSSOUT: [PA4]);

/*
The sub-GHz radio SPI clock is derived from the PCLK3 clock. The SUBGHZSPI_SCK
frequency is obtained by PCLK3 divided by two. The SUBGHZSPI_SCK clock maximum
speed must not exceed 16 MHz.
*/
