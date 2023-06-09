use embassy_stm32::{
    gpio::{Input, Pull},
    peripherals::{PA4, PA5, PA6, PA7},
};

#[derive(Clone, Copy)]
pub enum Key {
    S2,
    S3,
    S4,
    S5,
}

pub struct Keys<'d> {
    s2: Input<'d, PA4>,
    s3: Input<'d, PA5>,
    s4: Input<'d, PA6>,
    s5: Input<'d, PA7>,
    pressed: Option<Key>,
}

impl Keys<'_> {
    // pub fn scan(&mut self, timer: &mut SysCounterUs) -> Option<Key> {
    //     if let Some(pressed_key) = self.pressed {
    //         if match pressed_key {
    //             Key::S2 => self.s2.is_high(),
    //             Key::S3 => self.s3.is_high(),
    //             Key::S4 => self.s4.is_high(),
    //             Key::S5 => self.s5.is_high(),
    //         } {
    //             self.pressed = None;
    //         }
    //         None
    //     } else {
    //         self.pressed = if self.s2.is_low() {
    //             timer.start(50.micros()).unwrap();
    //             block!(timer.wait()).unwrap();
    //             Some(Key::S2)
    //         } else if self.s3.is_low() {
    //             timer.start(50.micros()).unwrap();
    //             block!(timer.wait()).unwrap();
    //             Some(Key::S3)
    //         } else if self.s4.is_low() {
    //             timer.start(50.micros()).unwrap();
    //             block!(timer.wait()).unwrap();
    //             Some(Key::S4)
    //         } else if self.s5.is_low() {
    //             timer.start(50.micros()).unwrap();
    //             block!(timer.wait()).unwrap();
    //             Some(Key::S5)
    //         } else {
    //             None
    //         };
    //         self.pressed
    //     }
    // }

    pub fn new(pa4: PA4, pa5: PA5, pa6: PA6, pa7: PA7) -> Self {
        Keys {
            s2: Input::new(pa4, Pull::Up),
            s3: Input::new(pa5, Pull::Up),
            s4: Input::new(pa6, Pull::Up),
            s5: Input::new(pa7, Pull::Up),
            pressed: None,
        }
    }
}
