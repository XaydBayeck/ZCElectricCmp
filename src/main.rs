#![deny(unsafe_code)]
#![no_std]
#![no_main]

use liquidled_testrs::{board::Board, keys::Key};
use nb::block;
use panic_halt as _;

use cortex_m_rt::entry;

use stm32f1xx_hal::{pac, prelude::*};

use liquidled_testrs::segements::{SEG_NUMS, WS};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Initial Board
    let Board {
        mut led,
        mut keys,
        mut segmts,
        mut timer,
    } = Board::new(dp, cp);

    timer.start(72.micros()).unwrap();
    block!(timer.wait()).unwrap();

    // let mut led_on = false;
    // Wait for the timer to trigger an update and change the state of the LED
    let mut number = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut num_sel = 2;
    let mut ws = WS::W0;
    let mut duan = 0;
    let mut led_on = false;
    loop {
        // LED test
        led_on = !led_on;
        if led_on {
            led.code_opt(Some(true), None, None);
        } else {
            led.code_opt(Some(false), None, None);
        }
        // Fresh segements
        segmts.display(SEG_NUMS[number[duan]], &mut timer);
        segmts.select(ws);
        segmts.fresh(&mut timer);
        ws = ws.next();
        duan = if duan >= 7 { 0 } else { duan + 1 };
        // key scan
        if let Some(key) = keys.scan(&mut timer) {
            match key {
                Key::S2 => {
                    let num = &mut number[num_sel];
                    *num = if *num >= 16 {
                        0
                    }else {
                        *num + 1
                    }
                },
                Key::S3 => num_sel = if num_sel >= 7 { 0 } else { num_sel + 1 },
                Key::S4 => {
                    let num = &mut number[num_sel];
                    *num = if *num <= 0 {
                        16
                    }else {
                        *num - 1
                    }
                },
                Key::S5 => num_sel = if num_sel <= 0 { 7 } else { num_sel - 1 },
            }
            timer.start(1950.micros()).unwrap();
        } else {
            timer.start(2.millis()).unwrap();
        }
        block!(timer.wait()).unwrap();
    }
}
