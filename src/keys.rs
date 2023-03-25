use stm32f1xx_hal::gpio::{gpioa::Parts, Input, PullUp, PA4, PA5, PA6, PA7};

use crate::utils::Gen;

pub struct Keys {
    s2: PA4<Input<PullUp>>,
    s3: PA5<Input<PullUp>>,
    s4: PA6<Input<PullUp>>,
    s5: PA7<Input<PullUp>>,
}

impl Gen<Keys> for Parts {
    fn get(mut self) -> Keys {
        Keys {
            s2: self.pa4.into_pull_up_input(&mut self.crl),
            s3: self.pa5.into_pull_up_input(&mut self.crl),
            s4: self.pa6.into_pull_up_input(&mut self.crl),
            s5: self.pa7.into_pull_up_input(&mut self.crl),
        }
    }
}
