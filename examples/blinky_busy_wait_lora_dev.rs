#![no_std]
#![no_main]

mod defmt_impls;

use cortex_m::asm::nop;
use hal::pac;
use hal::prelude::*;
use stm32wlxx_hal as hal;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

    // blinks the PB5 LED (located close to the usb connector)
    let mut led = gpiob
        .pa5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    loop {
        led.set_high().unwrap();
        for _ in 0..100_000 {
            nop();
        }
        led.set_low().unwrap();
        for _ in 0..100_000 {
            nop();
        }
    }
}
