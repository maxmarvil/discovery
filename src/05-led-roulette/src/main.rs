#![deny(unsafe_code)]
#![no_main]
#![no_std]

use aux5::{entry, Delay, DelayMs, LedArray, OutputSwitch};

#[entry]
fn main() -> ! {
    let (mut delay, mut leds): (Delay, LedArray) = aux5::init();

    let period = 500_u16;

    let led_count: usize = 8;
    let mut current: usize = 0;
    let mut next: usize = 1;
    let mut prev: usize = 7;

    loop {
        
        //step 1
        leds[current].on().ok();
        leds[prev].on().ok();
        delay.delay_ms(period);
        
        //step2
        leds[prev].off().ok();
        delay.delay_ms(period);
        
        current = update_pin(current, led_count);
        prev = update_pin(prev, led_count);
    }
}

fn update_pin (mut led: usize, count: usize) -> usize{
    if (led+1) < count {
        led +=1;
    } else {
        led = 0;
    }
    led
}
