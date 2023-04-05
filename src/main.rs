#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::{main, task, Spawner};
use embassy_stm32::{peripherals::*, Config, time::Hertz};
use embassy_time::{Duration, Timer};
use liquidled_testrs::{
    led::{Led, LedState},
    segements::Segments,
};

use {defmt_rtt as _, panic_probe as _};

#[task]
async fn blink(pc7: PC7, pc8: PC8, pc9: PC9) {
    let mut led = Led::new(pc7, pc8, pc9);
    let mut led_state = LedState::LHH;

    loop {
        led.set(led_state);
        led_state = led_state.flow_down();
        Timer::after(Duration::from_secs(1)).await
    }
}

#[task]
async fn display(
    pb3: PB3,
    pb4: PB4,
    pb5: PB5,
    pc10: PC10,
    pc11: PC11,
    pc12: PC12,
    fresh_frq: Duration,
) {
    let mut segmts = Segments::new(pb3, pb4, pb5, pc10, pc11, pc12, fresh_frq);

    let numbers = [0, 2, 1, 4, 3, 2, 6, 7];

    loop {
        info!("{:?}", segmts.current_state());
        Timer::after(segmts.state_exe(&numbers)).await;
        segmts.state_trans();
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sys_ck = Some(Hertz(72_000_000));
    config.rcc.hclk = Some(Hertz(72_000_000));
    config.rcc.pclk1 = Some(Hertz(36_000_000));
    config.rcc.pclk2 = Some(Hertz(72_000_000));

    let dp = embassy_stm32::init(config);

    spawner.spawn(blink(dp.PC7, dp.PC8, dp.PC9)).unwrap();

    let mut segmts = Segments::new(
        dp.PB3,
        dp.PB4,
        dp.PB5,
        dp.PC10,
        dp.PC11,
        dp.PC12,
        Duration::from_millis(2),
    );

    let numbers = [0, 2, 1, 4, 3, 2, 6, 7];

    loop {
        info!("{:?}", segmts.current_state());
        Timer::after(segmts.state_exe(&numbers)).await;
        segmts.state_trans();
    }
}
