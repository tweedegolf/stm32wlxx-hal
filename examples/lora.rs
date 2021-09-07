#![no_std]
#![no_main]

mod defmt_impls;

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

    let mut rcc = dp.RCC.constrain();
    let clocks = todo!();

    let gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    let sck = gpioa
        .pa5
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr)
        .into_af13(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa
        .pa6
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr)
        .into_af13(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa
        .pa7
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr)
        .into_af13(&mut gpioa.moder, &mut gpioa.afrl);
    let pins = (sck, miso, mosi, None::<()>);

    let mut subghzspi = Spi::subghzspi(
        dp.SPI3,
        pins,
        embedded_hal::spi::MODE_0,
        100.khz(),
        clocks,
        &mut rcc.apb3,
    );

    let nss = gpioa.pa4.into_push_pull_output_with_state(
        &mut gpioa.moder,
        &mut gpioa.otyper,
        gpio::State::High,
    );
}
