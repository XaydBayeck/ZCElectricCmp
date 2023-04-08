#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap, error};
use embassy_executor::{main, task, Spawner};
use embassy_stm32::{
    adc::Adc,
    dma::NoDma,
    i2c::{Error, I2c, TimeoutI2c},
    interrupt,
    pac::RCC,
    peripherals::*,
    rcc::low_level::RccPeripheral,
    time::Hertz,
    usart::Uart,
    Config,
};
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

#[task]
async fn usart(usart1: USART1, pa9: PA9, pa10: PA10, dma: (DMA1_CH4, DMA1_CH5)) {
    let irq = interrupt::take!(USART1);
    let mut usart = Uart::new(usart1, pa10, pa9, irq, dma.0, dma.1, Default::default());

    for i in 0..10 {
        Timer::after(Duration::from_secs(1)).await;
        info!("Before send message: {} s", i);
    }

    unwrap!(usart.write(b"Hello Embassy World!\r\n").await);
    info!("wote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.read(&mut buf).await);
        unwrap!(usart.write(&buf).await);
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    info!("Set JTag pins as normal pin.");
    AFIO::enable();
    unsafe {
        // RCC.apb2enr().modify(|w| w.set_afioen(true));
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

    spawner
        .spawn(usart(
            dp.USART1,
            dp.PA9,
            dp.PA10,
            (dp.DMA1_CH4, dp.DMA1_CH5),
        ))
        .unwrap();

    let irq = interrupt::take!(I2C1_EV);
    let mut i2c = I2c::new(
        dp.I2C1,
        dp.PB6,
        dp.PB7,
        irq,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );

    let mut timeout_i2c = TimeoutI2c::new(&mut i2c, Duration::from_millis(1000));

    let mut data = [0u8; 1];

    // loop {
        match timeout_i2c.blocking_write_read(0x40, &[0x3e], &mut data) {
            Ok(()) => info!("Whoami: {}", data[0]),
            Err(Error::Timeout) => error!("Operation timed out"),
            Err(e) => error!("I2c Error: {:?}", e),
        }
    // }
}
