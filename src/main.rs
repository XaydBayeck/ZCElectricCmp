#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [TIM3])]
mod app {
    use stm32f1xx_hal::{
        gpio::{
            gpioa::PA7,
            gpioc::{PC7, PC8, PC9},
            Edge, ExtiPin, GpioExt, Input, Output, PinState, PullUp, PushPull,
        },
        pac::TIM2,
        prelude::*,
        timer::MonoTimerUs,
    };

    // A monotonic timer to enable scheduling in RTIC
    #[monotonic(binds = TIM2, default = true)]
    type MyMono = MonoTimerUs<TIM2>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        button: PA7<Input<PullUp>>,
        led0: PC7<Output<PushPull>>,
        led1: PC8<Output<PushPull>>,
        led2: PC9<Output<PushPull>>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize the monotonic (SysTick rate in QEMU is 12 MHz)
        let rcc = ctx.device.RCC.constrain();
        let mut flash = ctx.device.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mono = ctx.device.TIM2.monotonic_us(&clocks);

        // let mut timer = ctx.device.TIM1.counter_us(&clocks);

        let mut gpioc = ctx.device.GPIOC.split();
        let led0 = gpioc
            .pc7
            .into_push_pull_output_with_state(&mut gpioc.crl, PinState::Low);
        let led1 = gpioc
            .pc8
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);
        let led2 = gpioc
            .pc9
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::High);

        let mut gpioa = ctx.device.GPIOA.split();
        let mut afio = ctx.device.AFIO.constrain();
        let mut button = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
        button.make_interrupt_source(&mut afio);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Rising);

        blink::spawn_after(1.secs()).unwrap();
        (
            Shared {},
            Local {
                button,
                led0,
                led1,
                led2,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led1, led2])]
    fn blink(ctx: blink::Context) {
        let led1 = ctx.local.led1;
        let led2 = ctx.local.led2;

        led1.toggle();
        led2.toggle();

        blink::spawn_after(1.secs()).unwrap();
    }

    #[task(binds = EXTI9_5, local = [button, led0])]
    fn button_click(ctx: button_click::Context) {
        ctx.local.button.clear_interrupt_pending_bit();
        ctx.local.led0.toggle();
    }
}
