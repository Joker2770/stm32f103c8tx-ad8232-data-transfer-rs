#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m::singleton;
use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    adc,
    dma::Half,
    pac,
    prelude::*,
    serial::{Config, Serial},
};
use unwrap_infallible::UnwrapInfallible;
// use rtt_target::{rprintln, rtt_init_print};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    // rtt_init_print!();
    // Acquire peripherals
    let p = pac::Peripherals::take().unwrap();
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

    let dma_ch1 = p.DMA1.split().1;

    // Setup ADC
    let adc1 = adc::Adc::adc1(p.ADC1, clocks);

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
        Config::default().baudrate(115200.bps()),
        &clocks,
    );

    // Configure pa0 as an analog input
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    let adc_dma = adc1.with_dma(adc_ch0, dma_ch1);
    let buf = singleton!(: [u16; 2] = [0; 2]).unwrap();

    let mut circ_buffer = adc_dma.circ_read(buf);

    while circ_buffer.readable_half().unwrap() != Half::First {}

    let _first_half = circ_buffer.peek(|half, _| *half).unwrap();

    while circ_buffer.readable_half().unwrap() != Half::Second {}

    let _second_half = circ_buffer.peek(|half, _| *half).unwrap();

    let (_buf, adc_dma) = circ_buffer.stop();
    //  rprintln!("{:?}", _buf);
    let (_adc1, _adc_ch0, _dma_ch1) = adc_dma.split();

    loop {
        let sent = singleton!(: [u8; 4] = [0; 4]).unwrap();
        sent[0] = 0xEC;
        sent[1] = _buf[1].to_be_bytes()[0];
        sent[2] = _buf[1].to_le_bytes()[0];
        sent[3] = (sent[1]) ^ (sent[2]);
        for i in sent {
            block!(serial.tx.write(*i)).unwrap_infallible();
        }
    }
}
