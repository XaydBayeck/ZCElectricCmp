use LedState::*;
use embassy_stm32::{gpio::{Output, Level::*, Speed::Medium}, peripherals::{PC7, PC8, PC9}};

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

pub struct Led<'d>(
    Output<'d, PC7>,
    Output<'d, PC8>,
    Output<'d, PC9>,
);

impl Led<'_> {
    pub fn set(&mut self, led_state: LedState) {
        led_state.set_led(self);
    }

    pub fn new(
        pc7: PC7,
        pc8: PC8,
        pc9: PC9,
    ) -> Self {
        Led(
            Output::new(pc7, Low, Medium),
            Output::new(pc8, Low, Medium),
            Output::new(pc9, Low, Medium),
        )
    }
}
