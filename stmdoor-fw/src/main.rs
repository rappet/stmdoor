#![no_main]
#![no_std]

extern crate alloc;

use cortex_m_rt::entry;
use embedded_alloc::Heap;
use onewire::{DeviceSearch, OneWire};
use panic_halt as _;
use stm32f0xx_hal as hal;

use alloc::format;
use core::fmt::Write;

use crate::hal::{delay::Delay, pac, prelude::*, serial::Serial};

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let mut p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(16.mhz()).freeze(&mut p.FLASH);

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiob = p.GPIOB.split(&mut rcc);

    // changed after Rev. 1.2
    let mut led1 = cortex_m::interrupt::free(|cs| gpiob.pb6.into_push_pull_output(cs));
    let mut led2 = cortex_m::interrupt::free(|cs| gpiob.pb5.into_push_pull_output(cs));
    let mut led3 = cortex_m::interrupt::free(|cs| gpiob.pb4.into_push_pull_output(cs));

    /*
    // available since Rev. 1.2
    let mut _ext1 = cortex_m::interrupt::free(|cs| gpiob.pb9.into_push_pull_output(cs));
    let mut _ext2 = cortex_m::interrupt::free(|cs| gpiob.pb8.into_push_pull_output(cs));
    let mut _ext3 = cortex_m::interrupt::free(|cs| gpiob.pb7.into_push_pull_output(cs));
    let mut _ext4 = cortex_m::interrupt::free(|cs| gpiob.pb6.into_push_pull_output(cs));
    */

    let mut dir = cortex_m::interrupt::free(|cs| gpioa.pa4.into_push_pull_output(cs));
    dir.set_high().ok();

    let (tx, rx) = cortex_m::interrupt::free(move |cs| {
        (
            gpioa.pa2.into_alternate_af1(cs),
            gpioa.pa3.into_alternate_af1(cs),
        )
    });
    let mut serial = Serial::usart2(p.USART2, (tx, rx), 9600.bps(), &mut rcc);

    let mut delay = Delay::new(cp.SYST, &rcc);

    let mut one_wire_pin = cortex_m::interrupt::free(|cs| gpiob.pb12.into_open_drain_output(cs));
    let mut one_wire = OneWire::new(&mut one_wire_pin, false);

    if one_wire.reset(&mut delay).is_err() {
        loop {}
    }

    led1.set_high().ok();

    loop {
        led2.set_high().ok();
        let mut search = DeviceSearch::new();
        while let Some(device) = one_wire.search_next(&mut search, &mut delay).unwrap() {
            led3.set_high().ok();
            let address = &device.address;

            let formatted = format!("one-wire-addr {address:?}\n");
            serial.write_str(&formatted).ok();

            delay.delay_ms(100_u16);
            led3.set_low().ok();
        }
        led2.set_low().ok();

        delay.delay_ms(100_u16);
    }
}
