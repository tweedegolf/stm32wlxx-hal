//! Serial Peripheral Interface (SPI) bus

use core::ptr;

use crate::{
    hal::spi::{FullDuplex, Mode, Phase, Polarity},
    rcc::{APB1_1, APB2},
};

use crate::gpio::*;
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
/// NSS Pin. This trait is sealed and cannot be implemented.
pub trait NssPin<SPI>: private::Sealed {}

impl private::Sealed for () {}
impl<SPI> NssPin<SPI> for () {}

macro_rules! pins {
    ($spi:ident,
     $af:ident,
     SCK: [$($sck:ident),*],
     MISO: [$($miso:ident),*],
     MOSI: [$($mosi:ident),*],
     NSS: [$($nss:ident),*]$(,)?
    ) => {
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
            impl private::Sealed for $nss<Alternate<$af, Output<PushPull>>> {}
            impl NssPin<$spi> for $nss<Alternate<$af, Output<PushPull>>> {}
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
            impl<SCK, MISO, MOSI, NSS> Spi<$SPIX, (SCK, MISO, MOSI, Option<NSS>)> {
                /// Configures the SPI peripheral to operate in full duplex master mode
                pub fn $spiX<F>(
                    spi: $SPIX,
                    pins: (SCK, MISO, MOSI, Option<NSS>),
                    mode: Mode,
                    freq: F,
                    clocks: Clocks,
                    apb: &mut $APBX,
                ) -> Self
                where
                    F: Into<Hertz>,
                    SCK: SckPin<$SPIX>,
                    MISO: MisoPin<$SPIX>,
                    MOSI: MosiPin<$SPIX>,
                    NSS: NssPin<$SPIX>,
                {
                    // enable or reset $SPIX
                    apb.enr().modify(|_, w| w.$spiXen().set_bit());
                    apb.rstr().modify(|_, w| w.$spiXrst().set_bit());
                    apb.rstr().modify(|_, w| w.$spiXrst().clear_bit());

                    // FRXTH: RXNE event is generated if the FIFO level is greater than or equal to
                    //        8-bit
                    // DS: 8-bit data size
                    // SSOE: Slave Select output enabled if NSS pin is passed
                    spi.cr2
                        .write(|w| unsafe {
                            w.frxth().set_bit().ds().bits(0b111).ssoe().bit(pins.3.is_some())
                        });

                    let br = Self::compute_baud_rate(clocks.$pclkX(), freq.into());

                    // CPHA: phase
                    // CPOL: polarity
                    // MSTR: master mode
                    // BR: 1 MHz
                    // SPE: SPI disabled
                    // LSBFIRST: MSB first
                    // SSM: enable hardware slave management if NSS pin is passed
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
                            .bit(pins.3.is_none())
                            .crcen()
                            .clear_bit()
                            .bidimode()
                            .clear_bit()
                    });

                    // spi.cr2.write(|w|
                        // w.ssoe()
                        // .bit(pins.3.is_some())
                        // .nssp()
                        // .bit(pins.3.is_some())
                    // );

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
                pub fn free(self) -> ($SPIX, (SCK, MISO, MOSI, Option<NSS>)) {
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

// SPI1
use crate::pac::SPI1;

hal! {
    SPI1: (spi1, APB2, spi1en, spi1rst, pclk1),
}

pins!(SPI1, AF5,
    SCK: [PA1, PA5, PB3],
    MISO: [PA6, PB4],
    MOSI: [PA7, PA12, PB5],
    NSS: [PA4, PA15, PB2],
);

// SPI2S2
use crate::pac::SPI2 as SPI2S2;

hal! {
    SPI2S2: (spi2s2, APB1_1, spi2s2en, spi2s2rst, pclk1),
}

pins!(SPI2S2, AF5,
    SCK: [PA8, PA9, PB10, PB13],
    MISO: [PA11, PB14],
    MOSI: [PA10, PB15],
    NSS: [PB9, PB12],
);

pins!(SPI2S2, AF3,
    SCK: [],
    MISO: [PA5],
    MOSI: [],
    NSS: [PA9],
);

// SUBGHZSPI
use crate::pac::SPI3 as SUBGHZSPI;

hal! {
    SUBGHZSPI: (subghzspi, APB3, subghzspien, subghzspirst, pclk1),
}

pins!(
    SUBGHZSPI,
    AF13,
    SCK: [PA5],
    MISO: [PA6],
    MOSI: [PA7],
    NSS: [PA4]
);
