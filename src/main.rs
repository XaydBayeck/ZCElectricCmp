#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

use panic_halt as _;

// #[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [TIM3])]
#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use liquidled_testrs::led::{Led, LedState};
    use stm32f1xx_hal::{
        gpio::GpioExt,
        pac::TIM3,
        // pac::TIM2,
        prelude::*,
        timer::{CounterUs, Event},
    };

    // A monotonic timer to enable scheduling in RTIC
    // #[monotonic(binds = TIM2, default = true)]
    // type MyMono = MonoTimerUs<TIM2>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led_state: LedState,
        led: Led,
        // segmts: Segments,
        timer_handler: CounterUs<TIM3>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let gpioa = ctx.device.GPIOA.split();
        let gpiob = ctx.device.GPIOB.split();
        let mut gpioc = ctx.device.GPIOC.split();
        let mut afio = ctx.device.AFIO.constrain();

        let (pb3, pb4, _pb5) = (gpiob.pb3, gpiob.pb4, gpiob.pb5);
        let pa15 = gpioa.pa15;
        let (_pa15, _pb3, _pb4) = afio.mapr.disable_jtag(pa15, pb3, pb4);
        //
        // let segmts = Segments::new(
        //     pb3,
        //     pb4,
        //     pb5,
        //     &mut gpiob.crl,
        //     gpioc.pc10,
        //     gpioc.pc11,
        //     gpioc.pc12,
        //     &mut gpioc.crh,
        //     1.secs(),
        // );

        let mut led = Led::new(
            gpioc.pc7,
            gpioc.pc8,
            gpioc.pc9,
            &mut gpioc.crl,
            &mut gpioc.crh,
        );
        led.set(LedState::LHH);

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
            .hclk(36.MHz())
            .pclk1(36.MHz())
            .pclk2(72.MHz())
            .adcclk(36.MHz())
            .freeze(&mut flash.acr);
        // let mono = ctx.device.TIM2.monotonic_us(&clocks);
        let mut timer = ctx.device.TIM3.counter_us(&clocks);
        timer.start(1.secs()).unwrap();
        timer.listen(Event::Update);

        // blink::spawn_after(1.secs()).unwrap();
        (
            Shared {},
            Local {
                led_state: LedState::LHH,
                led,
                // segmts,
                timer_handler: timer,
            },
            // init::Monotonics(mono),
            init::Monotonics(),
        )
    }

    // #[task(local = [led_state, led])]
    // fn blink(ctx: blink::Context) {
    //     let led_state = ctx.local.led_state;
    //     ctx.local.led.set(*led_state);
    //     *led_state = led_state.flow_down();
    //
    //     blink::spawn_after(1.secs()).unwrap();
    // }

    #[task(binds=TIM3, priority=1,local = [timer_handler, led_state, led])]
    fn display(ctx: display::Context) {
        let led_state = ctx.local.led_state;
        ctx.local.led.set(*led_state);
        *led_state = led_state.flow_down();
        let timer = ctx.local.timer_handler;
        timer.start(1.secs()).unwrap();
        //
        // timer
        //     .start(segmts.state_exe([0, 2, 1, 4, 3, 2, 6, 7]))
        //     .unwrap();
        //
        // segmts.state_trans();

        timer.clear_interrupt(Event::Update);
    }
}
