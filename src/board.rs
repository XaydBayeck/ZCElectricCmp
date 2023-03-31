use stm32f1xx_hal::{
    device::Peripherals,
    prelude::{
        _fugit_ExtU32, _stm32_hal_afio_AfioExt, _stm32_hal_flash_FlashExt, _stm32_hal_gpio_GpioExt,
    },
    rcc::RccExt,
    timer::{SysCounterUs, Timer},
};

use crate::{keys::Keys, led::Led, segements::Segments};

pub struct Board {
    pub led: Led,
    pub keys: Keys,
    pub segmts: Segments,
    pub timer: SysCounterUs,
}

impl Board {
    pub fn new(dp: Peripherals, cp: cortex_m::Peripherals) -> Self {
        let mut gpioa = dp.GPIOA.split();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();

        let (pb3, pb4, pb5) = (gpiob.pb3, gpiob.pb4, gpiob.pb5);
        let pa15 = gpioa.pa15;
        let (_pa15, pb3, pb4) = dp.AFIO.constrain().mapr.disable_jtag(pa15, pb3, pb4);

        let segmts = Segments::new(
            pb3,
            pb4,
            pb5,
            &mut gpiob.crl,
            gpioc.pc10,
            gpioc.pc11,
            gpioc.pc12,
            &mut gpioc.crh,
            2.millis(),
        );

        let led = Led::new(
            gpioc.pc7,
            gpioc.pc8,
            gpioc.pc9,
            &mut gpioc.crl,
            &mut gpioc.crh,
        );

        // keys
        // let key_up = gpioa.pa4.into_pull_up_input(&mut gpioa.crl);
        // let key_down = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let keys = Keys::new(gpioa.pa4, gpioa.pa5, gpioa.pa6, gpioa.pa7, &mut gpioa.crl);

        // Take ownership over the raw flash and rcc devices and convert them into the corresponding
        // HAL structs
        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
        // `clocks`
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let timer = Timer::syst(cp.SYST, &clocks).counter_us();

        Self {
            led,
            keys,
            segmts,
            timer,
        }
    }
}
