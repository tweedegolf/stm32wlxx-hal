#![no_std]
#![no_main]

mod defmt_impls;

use cortex_m::asm::nop;
use stm32wlxx_hal as hal;
use hal::pac;
use hal::prelude::*;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    let mut led = gpioa
        .pa4
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

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