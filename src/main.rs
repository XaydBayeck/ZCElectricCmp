// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#![no_main]

use panic_halt as _;

// #[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [TIM3])]
#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use liquidled_testrs::led::{Led, LedState};
    use liquidled_testrs::segements::Segments;
    use stm32f1xx_hal::adc;
    use stm32f1xx_hal::device::ADC1;
    use stm32f1xx_hal::gpio::{Analog, PB1};
    use stm32f1xx_hal::{
        gpio::GpioExt,
        pac::Interrupt,
        pac::TIM1,
        pac::TIM2,
        prelude::*,
        timer::{CounterUs, Event},
    };

    // A monotonic timer to enable scheduling in RTIC
    // #[monotonic(binds = TIM2, default = true)]
    // type MyMono = MonoTimerUs<TIM2>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {
        display_nums: [usize; 8],
    }

    #[local]
    struct Local {
        led_state: LedState,
        led: Led,
        segmts: Segments,
        timer_handler1: CounterUs<TIM1>,
        timer_handler2: CounterUs<TIM2>,
        adc1: adc::Adc<ADC1>,
        ch: PB1<Analog>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::TIM1_UP);

        let gpioa = ctx.device.GPIOA.split();
        let mut gpiob = ctx.device.GPIOB.split();
        let mut gpioc = ctx.device.GPIOC.split();
        let mut afio = ctx.device.AFIO.constrain();

        let (pb3, pb4, pb5) = (gpiob.pb3, gpiob.pb4, gpiob.pb5);
        let pa15 = gpioa.pa15;
        let (_pa15, pb3, pb4) = afio.mapr.disable_jtag(pa15, pb3, pb4);

        let segmts = Segments::new(
            pb3,
            pb4,
            pb5,
            &mut gpiob.crl,
            gpioc.pc10,
            gpioc.pc11,
            gpioc.pc12,
            &mut gpioc.crh,
            2000.micros(),
        );

        let mut led = Led::new(
            gpioc.pc7,
            gpioc.pc8,
            gpioc.pc9,
            &mut gpioc.crl,
            &mut gpioc.crh,
        );
        led.set(LedState::HLH);

        // let mut button = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
        // button.make_interrupt_source(&mut afio);
        // button.enable_interrupt(&mut ctx.device.EXTI);
        // button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Rising);

        let rcc = ctx.device.RCC.constrain();
        let mut flash = ctx.device.FLASH.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(72.MHz())
            // .hclk(36.MHz())
            // .pclk1(36.MHz())
            // .pclk2(72.MHz())
            // .adcclk(36.MHz())
            .freeze(&mut flash.acr);
        // let mono = ctx.device.TIM2.monotonic_us(&clocks);
        let mut timer = ctx.device.TIM1.counter_us(&clocks);
        // timer.start(1.secs()).unwrap();
        timer.listen(Event::Update);

        let mut timer2 = ctx.device.TIM2.counter_us(&clocks);
        timer2.listen(Event::Update);

        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
        }

        // NOTE ADC settings
        let adc1 = adc::Adc::adc1(ctx.device.ADC1, clocks);
        let ch = gpiob.pb1.into_analog(&mut gpiob.crl);

        // blink::spawn_after(1.secs()).unwrap();
        (
            Shared {
                display_nums: [18, 0, 0, 0, 0, 0, 0, 0],
            },
            Local {
                led_state: LedState::LHH,
                led,
                segmts,
                timer_handler1: timer,
                timer_handler2: timer2,
                adc1,
                ch,
            },
            // init::Monotonics(mono),
            init::Monotonics(),
        )
    }

    #[idle(shared = [display_nums],local= [adc1, ch])]
    fn idle(mut ctx: idle::Context) -> ! {
        let adc1 = ctx.local.adc1;
        let ch = ctx.local.ch;

        let mut data: usize;
        loop {
            data = adc1.read(ch).unwrap();
            data = data * 5;
            let data1 = data / 2118;
            data = 10 * (data - 2118 * data1);
            let data2 = data / 2118;
            data = 10 *(data - 2118 * data2);
            let data3 = data / 2118;
            ctx.shared.display_nums.lock(|nums| {
                nums[1] = data1;
                nums[2] = data2;
                nums[3] = data3
            })
        }
    }
    // #[task(local = [led_state, led])]
    // fn blink(ctx: blink::Context) {
    //     let led_state = ctx.local.led_state;
    //     ctx.local.led.set(*led_state);
    //     *led_state = led_state.flow_down();
    //
    //     blink::spawn_after(1.secs()).unwrap();
    // }
    //
    #[task(binds=TIM2, local = [timer_handler2, led_state, led, counter:u8 = 0])]
    fn blink(ctx: blink::Context) {
        let timer = ctx.local.timer_handler2;
        // timer.clear_interrupt(Event::Update);

        let counter = ctx.local.counter;
        if *counter >= 30 {
            *counter = 0;
            let led_state = ctx.local.led_state;
            ctx.local.led.set(*led_state);
            *led_state = led_state.flow_down();
        } else {
            *counter += 1;
        }

        timer.start(10.millis()).unwrap();
    }

    #[task(binds=TIM1_UP, shared = [display_nums], local = [timer_handler1, segmts])]
    fn display(mut ctx: display::Context) {
        let timer = ctx.local.timer_handler1;
        // timer.clear_interrupt(Event::Update);

        let segmts = ctx.local.segmts;

        ctx.shared.display_nums.lock(|nums| {
            timer
                // .start(segmts.state_exe([0, 2, 1, 4, 3, 2, 6 + 18, 7]))
                .start(segmts.state_exe(nums))
                .unwrap();
        });

        segmts.state_trans();
    }
}
