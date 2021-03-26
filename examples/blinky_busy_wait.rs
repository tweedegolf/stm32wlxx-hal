#![no_std]
#![no_main]

mod defmt_impls;

use stm32wlxx_hal as hal;
use hal::pac;

use cortex_m_rt::entry;
use defmt_impls::exit;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // TODO initialize gpio pin PA4
    // and have it blink

    exit();
}