#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{time::Hertz, Config};
use embassy_time::{Timer, Duration};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(Hertz(36_000_000));
    let _p = embassy_stm32::init(config);

    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await
    }
}
