#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::ops::BitXor;

// Halt on panic
use panic_halt as _;

// use core::fmt::Write; // for pretty formatting of the output

// use rtt_target::{rprintln, rtt_init_print};

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    adc, pac,
    prelude::*,
    serial::{Config, Serial},
};
use unwrap_infallible::UnwrapInfallible;

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    // rtt_init_print!();
    // Acquire peripherals
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(56.MHz())
        .pclk1(28.MHz())
        .adcclk(14.MHz())
        .freeze(&mut flash.acr);
    /*
    // Alternative configuration using dividers and multipliers directly
    let clocks = rcc.cfgr.freeze_with_config(rcc::Config {
        hse: Some(8_000_000),
        pllmul: Some(7),
        hpre: rcc::HPre::DIV1,
        ppre1: rcc::PPre::DIV2,
        ppre2: rcc::PPre::DIV1,
        usbpre: rcc::UsbPre::DIV1_5,
        adcpre: rcc::AdcPre::DIV2,
    }, &mut flash.acr);*/

    // Setup ADC
    let mut adc1 = adc::Adc::adc1(p.ADC1, clocks);

    // Prepare the alternate function I/O registers
    let mut afio = p.AFIO.constrain();

    // Setup GPIOA
    let mut gpioa = p.GPIOA.split();

    // USART1
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    // Set up the usart device. Take ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    let mut serial = Serial::new(
        p.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(921600.bps()),
        &clocks,
    );

    // Configure pa0 as an analog input
    let mut adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    //let mut delay = hal::timer::Timer::syst(cp.SYST, &clocks).delay();
    // or
    let mut delay = cp.SYST.delay(&clocks);

    loop {
        let data: u16 = adc1.read(&mut adc_ch0).unwrap();

        let sent: &mut [u8; 4] = &mut [0; 4];
        sent[0] = 0xEC;
        sent[1] = data.to_be_bytes()[0];
        sent[2] = data.to_le_bytes()[0];
        sent[3] = sent[1].bitxor(sent[2]);
        for c in sent {
            block!(serial.tx.write(*c)).unwrap_infallible();
        }
        delay.delay(100.millis());
    }
}
