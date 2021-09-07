#![no_std]
#![no_main]

mod defmt_impls;

use gpio::State;
use hal::prelude::*;
use hal::{gpio, pac, spi::Spi};
use pac::interrupt;
use stm32wlxx_hal as hal;

use cortex_m_rt::entry;

/*
 * Example that uses the SUBGHZSPI to communicate with the
 * Radio. This example is a work in progress and does not work yet.
 * The idea is to implement a ping-pong example, sending messages
 * at the EU868 RF band.
 */

#[interrupt]
// Radio IRQs RFBUSY interrupt through EXTI
fn RADIO_BUSY() {
    // Handle Radio IRQ's and RFBUSY events
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // TRY the other clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

    let sck = gpiob
        .pa13
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let miso = gpiob
        .pa14
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let mosi = gpioa
        .pa10
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr)
        .into_af5(&mut gpioa.moder, &mut gpioa.afrh);

    let pins = (sck, miso, mosi, None::<()>);

    let mut spi1 = Spi::spi2s2(
        dp.SPI2,
        pins,
        embedded_hal::spi::MODE_0,
        500.khz(),
        clocks,
        &mut rcc.apb1_1,
    );

    let mut nss = gpiob.pa9.into_push_pull_output_with_state(
        &mut gpiob.moder,
        &mut gpiob.otyper,
        State::High,
    );

    nss.set_low().unwrap();
    spi1.write(b"aaaaaaaa").unwrap();
    nss.set_high().unwrap();

    defmt_impls::exit();
}
