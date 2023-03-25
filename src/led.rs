use stm32f1xx_hal::gpio::{Output, PinState, PushPull, HL, PC7, PC8, PC9};

pub struct Led(
    PC7<Output<PushPull>>,
    PC8<Output<PushPull>>,
    PC9<Output<PushPull>>,
);

fn judge_state(pred: bool) -> PinState {
    if pred {
        PinState::Low
    } else {
        PinState::High
    }
}

impl Led {
    pub fn code(&mut self, light0: bool, light1: bool, light2: bool) {
        self.0.set_state(judge_state(light0));
        self.1.set_state(judge_state(light1));
        self.2.set_state(judge_state(light2));
    }

    pub fn code_opt(&mut self, light0: Option<bool>, light1: Option<bool>, light2: Option<bool>) {
        let judge_state = |pred: bool| if pred { PinState::Low } else { PinState::High };
        light0.map(|pred| self.0.set_state(judge_state(pred)));
        light1.map(|pred| self.1.set_state(judge_state(pred)));
        light2.map(|pred| self.2.set_state(judge_state(pred)));
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
