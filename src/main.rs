#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::{main, task, Spawner};
use embassy_stm32::{adc::Adc, pac::RCC, peripherals::*, time::Hertz, Config};
use embassy_time::{Delay, Duration, Timer};
use liquidled_testrs::{
    led::{Led, LedState},
    segements::{self, Segments},
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
    adc1: ADC1,
    mut pb1: PB1,
    pb3: PB3,
    pb4: PB4,
    pb5: PB5,
    pc10: PC10,
    pc11: PC11,
    pc12: PC12,
    fresh_frq: Duration,
) {
    let mut segmts = Segments::new(pb3, pb4, pb5, pc10, pc11, pc12, fresh_frq);

    let mut adc = Adc::new(adc1, &mut Delay);

    let mut numbers = [18usize, 0, 0, 0, 0, 0, 0, 0];

    let mut vrefint = adc.enable_vref(&mut Delay);
    let vrefint_sample = adc.read(&mut vrefint);
    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/CD00161566.pdf
        // 5.3.4 Embedded reference voltage
        const VREFINT_MV: u32 = 1200; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    loop {
        if *segmts.current_state() == segements::State::Freshed {
            let v = convert_to_millivolts(adc.read(&mut pb1)) as usize;
            numbers[1] = v / 1000;
            numbers[2] = v / 100 - numbers[1] * 10;
            numbers[3] = v / 10 - (v / 100) * 10;
        }
        Timer::after(segmts.state_exe(&numbers)).await;
        segmts.state_trans();
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    info!("Set JTag pins as normal pin.");
    unsafe {
        RCC.apb2enr().modify(|w| w.set_afioen(true));
        embassy_stm32::pac::AFIO.mapr().modify(|w| {
            w.set_swj_cfg(2);
        });
        RCC.apb2enr().modify(|w| w.set_afioen(false));
    }

    let mut config = Config::default();
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sys_ck = Some(Hertz(72_000_000));
    config.rcc.hclk = Some(Hertz(72_000_000));
    config.rcc.pclk1 = Some(Hertz(36_000_000));
    config.rcc.pclk2 = Some(Hertz(72_000_000));

    let dp = embassy_stm32::init(config);

    info!("PC7, PC8, PC9 as LED Output Pins.");
    spawner.spawn(blink(dp.PC7, dp.PC8, dp.PC9)).unwrap();

    info!("ADC1 use to sample voltage from PB1");
    info!("PB3, PB4 and PB5 use to send segment data");
    info!("PC10, PC11 and PC12 use to drive segment fresh");
    spawner
        .spawn(display(
            dp.ADC1,
            dp.PB1,
            dp.PB3,
            dp.PB4,
            dp.PB5,
            dp.PC10,
            dp.PC11,
            dp.PC12,
            Duration::from_millis(2),
        ))
        .unwrap();

    loop {
        // info!("{:?}", segmts.current_state());
        Timer::after(Duration::from_secs(1)).await;
    }
}
