use nb::block;
use stm32f1xx_hal::gpio::{Output, PinState, HL, PB3, PB4, PB5, PC10, PC11, PC12};
use stm32f1xx_hal::prelude::_fugit_ExtU32;

use stm32f1xx_hal::timer::SysCounterUs;

use crate::utils::Gen;

pub const SEG_NUMS: [u8; 18] = [
    0xfc, 0x60, 0xda, 0xf2, 0x66, 0xb6, 0xbe, 0xe0, 0xfe, 0xf6, 0xee, 0x3e, 0x9c, 0x7a, 0x9e, 0x8e,
    0x01, 0x00,
];

#[derive(Clone, Copy)]
pub enum WS {
    W0,
    W1,
    W2,
    W3,
    W4,
    W5,
    W6,
    W7,
}

impl WS {
    pub fn next(self) -> Self {
        use WS::*;
        match self {
            WS::W0 => W1,
            WS::W1 => W2,
            WS::W2 => W3,
            WS::W3 => W4,
            WS::W4 => W5,
            WS::W5 => W6,
            WS::W6 => W7,
            WS::W7 => W0,
        }
    }
}

#[allow(non_snake_case)]
pub struct Segments {
    seg_select_pins: (PC10<Output>, PC11<Output>, PC12<Output>),
    SFTCLK: PB5<Output>,
    LCHCLK: PB4<Output>,
    SDI: PB3<Output>,
}

impl Segments {
    // TODO: implement it
    pub fn display(&mut self, num: u8, timer: &mut SysCounterUs) {
        for i in 0..8 {
            self.SDI.set_state(if (num >> i) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            });
            self.SFTCLK.set_low();
            timer.start(5.micros()).unwrap();
            block!(timer.wait()).unwrap();
            self.SFTCLK.set_high();
        }
    }

    pub fn select(&mut self, idx: WS) {
        use PinState::*;
        let (st12, st11, st10) = match idx {
            WS::W0 => (Low, Low, Low),
            WS::W1 => (Low, Low, High),
            WS::W2 => (Low, High, Low),
            WS::W3 => (Low, High, High),
            WS::W4 => (High, Low, Low),
            WS::W5 => (High, Low, High),
            WS::W6 => (High, High, Low),
            WS::W7 => (High, High, High),
        };

        let (pc10, pc11, pc12) = &mut self.seg_select_pins;
        pc10.set_state(st10);
        pc11.set_state(st11);
        pc12.set_state(st12);
    }

    pub fn fresh(&mut self, timer: &mut SysCounterUs) {
        self.LCHCLK.set_high();
        timer.start(5.micros()).unwrap();
        block!(timer.wait()).unwrap();
        self.LCHCLK.set_low();
    }
}

impl Gen<Segments>
    for (
        PB3,
        PB4,
        PB5,
        &mut <PB3 as HL>::Cr,
        PC10,
        PC11,
        PC12,
        &mut <PC10 as HL>::Cr,
    )
{
    fn get(self) -> Segments {
        let (pb3, pb4, pb5, mut crl, pc10, pc11, pc12, mut crh) = self;

        let pb3 = pb3.into_push_pull_output_with_state(&mut crl, PinState::Low);
        let pb4 = pb4.into_push_pull_output_with_state(&mut crl, PinState::High);
        let pb5 = pb5.into_push_pull_output_with_state(&mut crl, PinState::Low);

        let pc10 = pc10.into_push_pull_output_with_state(&mut crh, PinState::Low);
        let pc11 = pc11.into_push_pull_output_with_state(&mut crh, PinState::Low);
        let pc12 = pc12.into_push_pull_output_with_state(&mut crh, PinState::Low);

        Segments {
            SDI: pb3,
            LCHCLK: pb4,
            SFTCLK: pb5,
            seg_select_pins: (pc10, pc11, pc12),
        }
    }
}
