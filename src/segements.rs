use defmt::Format;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals::{PB3, PB4, PB5, PC10, PC11, PC12},
};
use embassy_time::Duration;
use State::*;

pub const SEG_NUMS: [u8; 34] = [
    0xfc, 0x60, 0xda, 0xf2, 0x66, 0xb6, 0xbe, 0xe0, 0xfe, 0xf6, 0xee, 0x3e, 0x9c, 0x7a, 0x9e, 0x8e,
    0x01, 0x00, 0xfd, 0x61, 0xdb, 0xf3, 0x67, 0xb7, 0xbf, 0xe1, 0xff, 0xf7, 0xef, 0x3f, 0x9d, 0x7b,
    0x9f, 0x8f,
];

#[derive(Debug, Format, Clone, Copy, PartialEq)]
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

impl Into<usize> for WS {
    fn into(self) -> usize {
        match self {
            WS::W0 => 0,
            WS::W1 => 1,
            WS::W2 => 2,
            WS::W3 => 3,
            WS::W4 => 4,
            WS::W5 => 5,
            WS::W6 => 6,
            WS::W7 => 7,
        }
    }
}

impl Into<u8> for WS {
    fn into(self) -> u8 {
        match self {
            WS::W0 => 0,
            WS::W1 => 1,
            WS::W2 => 2,
            WS::W3 => 3,
            WS::W4 => 4,
            WS::W5 => 5,
            WS::W6 => 6,
            WS::W7 => 7,
        }
    }
}

#[derive(Debug, Format, Clone, Copy, PartialEq)]
pub enum State {
    Read(WS),
    Fresh,
    Freshed,
}

impl State {
    pub fn next(self) -> Self {
        match self {
            Read(WS::W7) => Fresh,
            Fresh => Freshed,
            Freshed => Read(WS::W0),
            Read(ws) => Read(ws.next()),
        }
    }
}

#[allow(non_snake_case)]
pub struct Segments<'d> {
    seg_select_pins: (Output<'d, PC10>, Output<'d, PC11>, Output<'d, PC12>),
    SFTCLK: Output<'d, PB5>,
    LCHCLK: Output<'d, PB4>,
    SDI: Output<'d, PB3>,
    fresh_sprt: Duration,
    ws: WS,
    state: State,
}

impl Segments<'_> {
    pub fn current_state(&self) -> &State {
        &self.state
    }

    pub fn state_exe(&mut self, numbers: &[usize; 8]) -> Duration {
        match self.state {
            State::Read(ds) => {
                let idx: usize = self.ws.into();
                self.read(SEG_NUMS[numbers[idx]], ds.into())
            }
            State::Fresh => self.fresh(),
            State::Freshed => self.freshed(),
        }
    }

    pub fn state_trans(&mut self) {
        if self.state == Fresh {
            self.ws = self.ws.next();
        }
        self.state = self.state.next();
    }

    pub fn read(&mut self, num: u8, ds: u8) -> Duration {
        self.SFTCLK.set_high();
        self.SDI.set_level(if (num >> ds) & 0x01 == 1 {
            Level::High
        } else {
            Level::Low
        });
        self.SFTCLK.set_low();
        Duration::from_micros(5)
    }

    pub fn select(&mut self, idx: WS) {
        use Level::*;
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
        pc10.set_level(st10);
        pc11.set_level(st11);
        pc12.set_level(st12);
    }

    pub fn fresh(&mut self) -> Duration {
        self.SFTCLK.set_high();
        self.select(self.ws);
        self.LCHCLK.set_high();
        Duration::from_micros(5)
    }

    pub fn freshed(&mut self) -> Duration {
        self.LCHCLK.set_low();
        self.fresh_sprt
    }

    pub fn new(
        pb3: PB3,
        pb4: PB4,
        pb5: PB5,
        pc10: PC10,
        pc11: PC11,
        pc12: PC12,
        fresh_frq: Duration,
    ) -> Self {
        let pb3 = Output::new(pb3, Level::Low, Speed::VeryHigh);
        let pb4 = Output::new(pb4, Level::High, Speed::VeryHigh);
        let pb5 = Output::new(pb5, Level::Low, Speed::VeryHigh);
        let pc10 = Output::new(pc10, Level::Low, Speed::VeryHigh);
        let pc11 = Output::new(pc11, Level::Low, Speed::VeryHigh);
        let pc12 = Output::new(pc12, Level::Low, Speed::VeryHigh);

        Segments {
            SDI: pb3,
            LCHCLK: pb4,
            SFTCLK: pb5,
            seg_select_pins: (pc10, pc11, pc12),
            fresh_sprt: fresh_frq - Duration::from_micros(40),
            ws: WS::W0,
            state: State::Freshed,
        }
    }
}
