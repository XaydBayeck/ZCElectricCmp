#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy_executor::{main, task, Spawner};
use embassy_stm32::{
    adc::Adc,
    interrupt,
    pac::RCC,
    peripherals::*,
    pwm::{
        self,
        simple_pwm::{PwmPin, SimplePwm},
    },
    rcc::low_level::RccPeripheral,
    time::{hz, Hertz},
    usart::Uart,
    Config,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};

use heapless::String;
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
    duty: &'static Mutex<CriticalSectionRawMutex, u16>,
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
            let v = convert_to_millivolts(adc.read(&mut pb1));
            *duty.lock().await = 1660 / v;

            let v = v as usize;
            numbers[1] = v / 1000;
            numbers[2] = v / 100 - numbers[1] * 10;
            numbers[3] = v / 10 - (v / 100) * 10;
        }
        Timer::after(segmts.state_exe(&numbers)).await;
        segmts.state_trans();
    }
}

// #[task]
// async fn usart(usart1: USART1, pa9: PA9, pa10: PA10, dma: (DMA1_CH4, DMA1_CH5)) {
//     let irq = interrupt::take!(USART1);
//     let mut usart = Uart::new(usart1, pa10, pa9, irq, dma.0, dma.1, Default::default());
//
//     unwrap!(usart.write(b"Hello Embassy World!\r\n").await);
//     info!("wote Hello, starting echo");
//
//     let mut buf = [0u8; 1];
//     loop {
//         unwrap!(usart.read(&mut buf).await);
//         unwrap!(usart.write(&buf).await);
//     }
// }

static DUTY: Mutex<CriticalSectionRawMutex, u16> = Mutex::new(1u16);

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
            &DUTY,
        ))
        .unwrap();

    // spawner
    //     .spawn(usart(
    //         dp.USART1,
    //         dp.PA9,
    //         dp.PA10,
    //         (dp.DMA1_CH4, dp.DMA1_CH5),
    //     ))
    //     .unwrap();

    spawner
        .spawn(adc_sender(
            dp.ADC2,
            dp.PC0,
            dp.USART1,
            dp.PA10,
            dp.PA9,
            dp.DMA1_CH4,
            dp.DMA1_CH5,
            &DUTY,
        ))
        .unwrap();

    let ch1 = PwmPin::new_ch1(dp.PA8);
    let mut pwm = SimplePwm::new(dp.TIM1, Some(ch1), None, None, None, hz(1));
    let max = pwm.get_max_duty();
    pwm.enable(pwm::Channel::Ch1);
    pwm.set_duty(pwm::Channel::Ch1, 0);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    loop {
        let duty_divied = *DUTY.lock().await;
        pwm.set_duty(
            pwm::Channel::Ch1,
            if duty_divied <= 1 || duty_divied == 0 {
                max - 1
            } else {
                max / if duty_divied > 100 { 100 } else { duty_divied }
            },
        );
        Timer::after(Duration::from_millis(10)).await;
        // pwm.set_duty(pwm::Channel::Ch1, 0);
        // Timer::after(Duration::from_secs(5)).await;
        //
        // pwm.set_duty(pwm::Channel::Ch1, max / 4);
        // Timer::after(Duration::from_secs(5)).await;
        //
        // pwm.set_duty(pwm::Channel::Ch1, max / 2);
        // Timer::after(Duration::from_secs(5)).await;
        //
        // pwm.set_duty(pwm::Channel::Ch1, max - 1);
        // Timer::after(Duration::from_secs(5)).await;
    }
}

#[task]
async fn adc_sender(
    adc2: ADC2,
    mut pc0: PC0,
    usart1: USART1,
    pa10: PA10,
    pa9: PA9,
    dma1_ch4: DMA1_CH4,
    dma1_ch5: DMA1_CH5,
    duty_divied: &'static Mutex<CriticalSectionRawMutex, u16>,
) {
    let mut adc = Adc::new(adc2, &mut Delay);

    let mut vrefint = adc.enable_vref(&mut Delay);
    let vrefint_sample = adc.read(&mut vrefint);
    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/CD00161566.pdf
        // 5.3.4 Embedded reference voltage
        const VREFINT_MV: u32 = 1200; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    let irq = interrupt::take!(USART1);
    let mut usart = Uart::new(
        usart1,
        pa10,
        pa9,
        irq,
        dma1_ch4,
        dma1_ch5,
        Default::default(),
    );

    unwrap!(usart.write(b"Hello Embassy World!\r\n").await);
    info!("wote Hello, starting echo");

    loop {
        let v = convert_to_millivolts(adc.read(&mut pc0));
        let vef = String::<8>::from(v);
        let duty = String::<8>::from(*duty_divied.lock().await);
        // info!("Get voltage: {}", v);
        unwrap!(usart.write(vef.as_bytes()).await);
        unwrap!(usart.write(b",").await);
        unwrap!(usart.write(duty.as_bytes()).await);
        unwrap!(usart.write(b"\r\n").await);
        Timer::after(Duration::from_millis(100)).await;
    }
}
