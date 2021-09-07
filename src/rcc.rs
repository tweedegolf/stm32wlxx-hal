//! Reset and Clock Control

use crate::flash::ACR;

use crate::stm32::{rcc, RCC};
use crate::time::Hertz;
use cast::u32;

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb2: AHB2 { _0: () },
            apb1_1: APB1_1 { _0: () },
            apb2: APB2 { _0: () },
            apb3: APB3 { _0: () },
            cfgr: CFGR {},
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus 2 (AHB2) registers
    pub ahb2: AHB2,
    /// AMBA Advanced Peripheral Bus 1 (APB1) register block 1
    pub apb1_1: APB1_1,
    /// AMBA Advanced Peripheral Bus 2 (APB2) registers
    pub apb2: APB2,
    /// AMBA Advanced Peripheral Bus 3 (APB3) registers
    pub apb3: APB3,
    /// Clock configuration register
    pub cfgr: CFGR,
}

/// AMBA High-performance Bus 2 (AHB2) registers
pub struct AHB2 {
    _0: (),
}

impl AHB2 {
    pub(crate) fn enr(&mut self) -> &rcc::AHB2ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ahb2enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::AHB2RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ahb2rstr }
    }
}

/// Advanced Peripheral Bus 1 (APB1) register block 1
pub struct APB1_1 {
    _0: (),
}

impl APB1_1 {
    pub(crate) fn enr(&mut self) -> &rcc::APB1ENR1 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb1enr1 }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB1RSTR1 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb1rstr1 }
    }
}

/// Advanced Peripheral Bus 2 (APB2) registers
pub struct APB2 {
    _0: (),
}

impl APB2 {
    pub(crate) fn enr(&mut self) -> &rcc::APB2ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB2RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2rstr }
    }
}

/// Advanced Peripheral Bus 3 (APB3) registers
pub struct APB3 {
    _0: (),
}

impl APB3 {
    pub(crate) fn enr(&mut self) -> &rcc::APB3ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb3enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB3RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb3rstr }
    }
}

const HSI: u32 = 16_000_000; // Hz

/// Clock configuration
pub struct CFGR {}

impl CFGR {
    pub fn freeze(&self, acr: &mut ACR) -> Clocks {
        // TODO this works, but does not allow for any
        // configurations in cfgr. Needs rewrite before
        // merging

        let rcc = unsafe { &*RCC::ptr() };

        let lsi_used = false;
        let (clock_speed, pll_source) = (HSI, PllSource::HSI16);
        if pll_source == PllSource::HSI16 {
            rcc.cr.write(|w| w.hsion().set_bit());
            while rcc.cr.read().hsirdy().bit_is_clear() {}
        }

        let sysclk = HSI;
        assert!(sysclk <= 80_000_000);

        let (hpre_bits, hpre_div) = (0b0000, 1);
        let hclk = sysclk / hpre_div;
        assert!(hclk <= sysclk);

        let (ppre1_bits, ppre1) = (0b000, 1u8);
        let pclk1 = hclk / u32(ppre1);
        assert!(pclk1 <= sysclk);

        let (ppre2_bits, ppre2) = (0b000, 1u8);
        let pclk2 = hclk / u32(ppre2);

        // adjust flash wait states
        unsafe {
            acr.acr().write(|w| {
                w.latency().bits(if hclk <= 16_000_000 {
                    0b000
                } else if hclk <= 32_000_000 {
                    0b001
                } else if hclk <= 48_000_000 {
                    0b010
                } else if hclk <= 64_000_000 {
                    0b011
                } else {
                    0b100
                })
            })
        }

        let sysclk_src_bits;
        {
            sysclk_src_bits = 0b01;

            rcc.cr.write(|w| w.hsion().set_bit());
            while rcc.cr.read().hsirdy().bit_is_clear() {}

            // SW: HSI selected as system clock
            rcc.cfgr.write(|w| unsafe {
                w.ppre2()
                    .bits(ppre2_bits)
                    .ppre1()
                    .bits(ppre1_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .bits(sysclk_src_bits)
            });
        }
        while rcc.cfgr.read().sws().bits() != sysclk_src_bits {}

        // MSI always starts on reset
        {
            rcc.cr
                .modify(|_, w| w.msion().clear_bit().msipllen().clear_bit())
        }

        Clocks {
            pclk1: Hertz(pclk1),
            pclk2: Hertz(pclk2),
            sysclk: Hertz(sysclk),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsiFreq {
    #[doc = "range 0 around 100 kHz"]
    RANGE100K = 0,
    #[doc = "range 1 around 200 kHz"]
    RANGE200K = 1,
    #[doc = "range 2 around 400 kHz"]
    RANGE400K = 2,
    #[doc = "range 3 around 800 kHz"]
    RANGE800K = 3,
    #[doc = "range 4 around 1 MHz"]
    RANGE1M = 4,
    #[doc = "range 5 around 2 MHz"]
    RANGE2M = 5,
    #[doc = "range 6 around 4 MHz"]
    RANGE4M = 6,
    #[doc = "range 7 around 8 MHz"]
    RANGE8M = 7,
    #[doc = "range 8 around 16 MHz"]
    RANGE16M = 8,
    #[doc = "range 9 around 24 MHz"]
    RANGE24M = 9,
    #[doc = "range 10 around 32 MHz"]
    RANGE32M = 10,
    #[doc = "range 11 around 48 MHz"]
    RANGE48M = 11,
}

impl MsiFreq {
    fn to_hertz(self) -> Hertz {
        Hertz(match self {
            Self::RANGE100K => 100_000,
            Self::RANGE200K => 200_000,
            Self::RANGE400K => 400_000,
            Self::RANGE800K => 800_000,
            Self::RANGE1M => 1_000_000,
            Self::RANGE2M => 2_000_000,
            Self::RANGE4M => 4_000_000,
            Self::RANGE8M => 8_000_000,
            Self::RANGE16M => 16_000_000,
            Self::RANGE24M => 24_000_000,
            Self::RANGE32M => 32_000_000,
            Self::RANGE48M => 48_000_000,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// PLL Source
pub enum PllSource {
    /// Multi-speed internal clock
    MSI,
    /// High-speed internal clock
    HSI16,
    /// High-speed external clock
    HSE,
}

impl PllSource {
    fn to_pllsrc(self) -> u8 {
        match self {
            Self::MSI => 0b01,
            Self::HSI16 => 0b10,
            Self::HSE => 0b11,
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    pclk1: Hertz,
    pclk2: Hertz,
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
