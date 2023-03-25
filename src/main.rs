#![deny(unsafe_code)]
#![no_std]
#![no_main]

use liquidled_testrs::{keys::Key, utils::Gen};
use nb::block;
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

use liquidled_testrs::segements::{SEG_NUMS, WS};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Initial segements
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let (pb3, pb4, pb5) = (gpiob.pb3, gpiob.pb4, gpiob.pb5);
    let pa15 = gpioa.pa15;
    let (_pa15, pb3, pb4) = dp.AFIO.constrain().mapr.disable_jtag(pa15, pb3, pb4);

    let mut gpioc = dp.GPIOC.split();
    let mut segemts = (
        pb3,
        pb4,
        pb5,
        &mut gpiob.crl,
        gpioc.pc10,
        gpioc.pc11,
        gpioc.pc12,
        &mut gpioc.crh,
    )
        .get();

    let mut led = gpioc.pc7.into_push_pull_output(&mut gpioc.crl);

    // keys
    // let key_up = gpioa.pa4.into_pull_up_input(&mut gpioa.crl);
    // let key_down = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
    let mut keys = (gpioa.pa4, gpioa.pa5, gpioa.pa6, gpioa.pa7, &mut gpioa.crl).get();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut timer = Timer::syst(cp.SYST, &clocks).counter_us();
    timer.start(72.micros()).unwrap();
    block!(timer.wait()).unwrap();

    // let mut led_on = false;
    // Wait for the timer to trigger an update and change the state of the LED
    let number = [
        [0, 2, 1, 4, 3, 2, 6, 1],
        [0, 2, 1, 4, 3, 2, 6, 7],
        [0, 2, 1, 4, 1, 7, 3, 6],
    ];
    let mut num_sel = 2;
    let mut ws = WS::W0;
    let mut duan = 0;
    let mut led_on = false;
    loop {
        // LED test
        led_on = !led_on;
        if led_on {
            led.set_low();
        } else {
            led.set_high();
        }
        // Fresh segements
        segemts.display(SEG_NUMS[number[num_sel][duan]] + 0x01, &mut timer);
        segemts.select(ws);
        segemts.fresh(&mut timer);
        ws = ws.next();
        duan = if duan >= 7 { 0 } else { duan + 1 };
        // key scan
        if let Some(key) = keys.scan(&mut timer) {
            match key {
                Key::S2 => num_sel = if num_sel >= 2 { 0 } else { num_sel + 1 },
                Key::S3 => num_sel = if num_sel >= 2 { 0 } else { num_sel + 1 },
                Key::S4 => num_sel = if num_sel <= 0 { 2 } else { num_sel - 1 },
                Key::S5 => num_sel = if num_sel <= 0 { 2 } else { num_sel - 1 },
            }
            timer.start(1950.micros()).unwrap();
        } else {
            timer.start(2.millis()).unwrap();
        }
        block!(timer.wait()).unwrap();
    }
}
