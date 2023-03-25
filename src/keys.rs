use nb::block;
use stm32f1xx_hal::prelude::_fugit_ExtU32;
use stm32f1xx_hal::{
    gpio::{Input, PullUp, HL, PA4, PA5, PA6, PA7},
    timer::SysCounterUs,
};


#[derive(Clone, Copy)]
pub enum Key {
    S2,
    S3,
    S4,
    S5,
}

pub struct Keys {
    s2: PA4<Input<PullUp>>,
    s3: PA5<Input<PullUp>>,
    s4: PA6<Input<PullUp>>,
    s5: PA7<Input<PullUp>>,
    pressed: Option<Key>,
}

impl Keys {
    pub fn scan(&mut self, timer: &mut SysCounterUs) -> Option<Key> {
        if let Some(pressed_key) = self.pressed {
            if match pressed_key {
                Key::S2 => self.s2.is_high(),
                Key::S3 => self.s3.is_high(),
                Key::S4 => self.s4.is_high(),
                Key::S5 => self.s5.is_high(),
            } {
                self.pressed = None;
            }
            None
        } else {
            self.pressed = if self.s2.is_low() {
                timer.start(50.micros()).unwrap();
                block!(timer.wait()).unwrap();
                Some(Key::S2)
            } else if self.s3.is_low() {
                timer.start(50.micros()).unwrap();
                block!(timer.wait()).unwrap();
                Some(Key::S3)
            } else if self.s4.is_low() {
                timer.start(50.micros()).unwrap();
                block!(timer.wait()).unwrap();
                Some(Key::S4)
            } else if self.s5.is_low() {
                timer.start(50.micros()).unwrap();
                block!(timer.wait()).unwrap();
                Some(Key::S5)
            } else {
                None
            };
            self.pressed
        }
    }

    pub fn new(pa4: PA4, pa5: PA5, pa6: PA6, pa7: PA7, crl: &mut <PA4 as HL>::Cr) -> Self {
        Keys {
            s2: pa4.into_pull_up_input(crl),
            s3: pa5.into_pull_up_input(crl),
            s4: pa6.into_pull_up_input(crl),
            s5: pa7.into_pull_up_input(crl),
            pressed: None,
        }
    }
}

