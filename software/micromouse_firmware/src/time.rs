use stm32f4xx_hal::stm32 as stm32f405;

static OVERFLOW_VALUE: u32 = 65535 / 4;

pub struct Time {
    timer: stm32f405::TIM1,
    last_time: u32,
    overflows: u32,
}

impl Time {
    pub fn setup(rcc: &stm32f405::RCC, timer: stm32f405::TIM1) -> Time {
        // Enable clock for timer 1
        rcc.apb2enr.modify(|_, w| w.tim1en().set_bit());

        // setup the timer

        // 16MHz ABP2
        //timer.psc.write(|w| w.psc().bits(8000));

        // 84MHz ABP2
        // Runs at 4 ticks every ms, so needs division later on
        timer.psc.write(|w| w.psc().bits(42000));

        timer.cr1.modify(|_, w| w.cen().set_bit());
        timer.cnt.write(|w| w.cnt().bits(0));

        Time {
            timer,
            last_time: 0,
            overflows: 0,
        }
    }

    #[inline(always)]
    pub fn now(&mut self) -> u32 {
        let current_time = self.timer.cnt.read().cnt().bits() as u32 / 4;

        if current_time < self.last_time {
            self.overflows += 1;
        }

        self.last_time = current_time;

        current_time + self.overflows * OVERFLOW_VALUE
    }

    pub fn delay(&mut self, msecs: u32) {
        let start_time = self.now();

        while self.now() - start_time < msecs {}
    }
}
