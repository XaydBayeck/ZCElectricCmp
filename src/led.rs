use stm32f1xx_hal::gpio::{Output, PinState, PushPull, HL, PC7, PC8, PC9};
use LedState::*;

#[derive(Debug, Clone, Copy)]
pub enum LedState {
    LLL,
    LLH,
    LHL,
    LHH,
    HLL,
    HLH,
    HHL,
    HHH,
}

impl LedState {
    pub fn flow_up(self) -> Self {
        match self {
            LLL => LLL,
            LLH => LHL,
            LHL => HLL,
            LHH => HHL,
            HLL => LLH,
            HLH => LHH,
            HHL => HLH,
            HHH => HHH,
        }
    }

    pub fn flow_down(self) -> Self {
        match self {
            LLL => LLL,
            LLH => HLL,
            LHL => LLH,
            LHH => HLH,
            HLL => LHL,
            HLH => HHL,
            HHL => LHH,
            HHH => HHH,
        }
    }

    pub fn set_led(self, led: &mut Led) {
        match self {
            LLL => {
                led.0.set_low();
                led.1.set_low();
                led.2.set_low();
            }
            LLH => {
                led.0.set_low();
                led.1.set_low();
                led.2.set_high();
            }
            LHL => {
                led.0.set_low();
                led.1.set_high();
                led.2.set_low();
            }
            LHH => {
                led.0.set_low();
                led.1.set_high();
                led.2.set_high();
            }
            HLL => {
                led.0.set_high();
                led.1.set_low();
                led.2.set_low();
            }
            HLH => {
                led.0.set_high();
                led.1.set_low();
                led.2.set_high();
            }
            HHL => {
                led.0.set_high();
                led.1.set_high();
                led.2.set_low();
            }
            HHH => {
                led.0.set_high();
                led.1.set_high();
                led.2.set_high();
            }
        }
    }
}

pub struct Led(
    PC7<Output<PushPull>>,
    PC8<Output<PushPull>>,
    PC9<Output<PushPull>>,
);

impl Led {
    pub fn code(&mut self, light0: PinState, light1: PinState, light2: PinState) {
        self.0.set_state(light0);
        self.1.set_state(light1);
        self.2.set_state(light2);
    }

    pub fn code_opt(
        &mut self,
        light0: Option<PinState>,
        light1: Option<PinState>,
        light2: Option<PinState>,
    ) {
        light0.map(|pred| self.0.set_state(pred));
        light1.map(|pred| self.1.set_state(pred));
        light2.map(|pred| self.2.set_state(pred));
    }

    pub fn set(&mut self, led_state: LedState) {
        led_state.set_led(self);
    }

    pub fn new(
        pc7: PC7,
        pc8: PC8,
        pc9: PC9,
        crl: &mut <PC7 as HL>::Cr,
        crh: &mut <PC8 as HL>::Cr,
    ) -> Self {
        Led(
            pc7.into_push_pull_output(crl),
            pc8.into_push_pull_output(crh),
            pc9.into_push_pull_output(crh),
        )
    }
}
