#![no_std]
#![no_main]

use core::cell::RefCell;
use core::panic::PanicInfo;

use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

use hal::gpio::PushPull;
use stm32f3xx_hal as hal;
use stm32f3xx_hal::{
    gpio::{Alternate, PA5, PA6, PA7},
    interrupt, pac,
    prelude::*,
    rcc::Clocks,
    spi::Spi,
    timer,
};

use rtt_target::{rprintln, rtt_init_print};

static TIMER: Mutex<RefCell<Option<timer::Timer<pac::TIM2>>>> = Mutex::new(RefCell::new(None));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("Panic: {:?}", info);
    loop {}
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("sanity");
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut dp.FLASH.constrain().acr);

    let mut timer = timer::Timer::new(dp.TIM2, clocks, &mut rcc.apb1);

    unsafe {
        cortex_m::peripheral::NVIC::unmask(timer.interrupt());
    }
    timer.enable_interrupt(timer::Event::Update);
    timer.start(75.milliseconds());
    cortex_m::interrupt::free(|cs| {
        TIMER.borrow(cs).replace(Some(timer));
    });

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let sck =
        gpioa
            .pa5
            .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let miso =
        gpioa
            .pa6
            .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mosi =
        gpioa
            .pa7
            .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mut spi = Spi::new(dp.SPI1, (sck, miso, mosi), 1.MHz(), clocks, &mut rcc.apb2);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut spi_cs = gpioe
        .pe3
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led0 = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led1 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led2 = gpioe
        .pe10
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led3 = gpioe
        .pe11
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led4 = gpioe
        .pe12
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led5 = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led6 = gpioe
        .pe14
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led7 = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    spi_cs.set_high().unwrap();
    spi_cs.set_low().unwrap();
    spi_cs.set_high().unwrap();
    spi_cs.set_low().unwrap();

    let msg_send: [u16; 1] = [0x8F00];
    let mut msg_sending = msg_send;
    rprintln!("msg_sending :: {:?}", msg_sending);
    rprintln!("msg_sending :: {:?}", (msg_sending[0] & 0xff00) as u8);
    rprintln!("msg_sending :: {:?}", (msg_sending[0] & 0x00ff) as u8);
    let msg_received = spi.transfer(&mut msg_sending).unwrap();
    rprintln!("msg_received :: {:?}", msg_received);
    rprintln!("msg_received :: {:?}", (msg_received[0] & 0xff00) as u8);
    rprintln!("msg_received :: {:?}", (msg_received[0] & 0x00ff) as u8);

    let mut count = 0;

    loop {
        match count {
            0 => {
                led0.set_high().unwrap();
                led4.set_low().unwrap();
            }
            1 => {
                led1.set_high().unwrap();
                led5.set_low().unwrap();
            }
            2 => {
                led2.set_high().unwrap();
                led6.set_low().unwrap();
            }
            3 => {
                led3.set_high().unwrap();
                led7.set_low().unwrap();
            }
            4 => {
                led4.set_high().unwrap();
                led0.set_low().unwrap();
            }
            5 => {
                led5.set_high().unwrap();
                led1.set_low().unwrap();
            }
            6 => {
                led6.set_high().unwrap();
                led2.set_low().unwrap();
            }
            7 => {
                led7.set_high().unwrap();
                led3.set_low().unwrap();
            }
            _ => {}
        }
        count += 1;
        if count > 7 {
            count = 0;
        }
        asm::wfi();
    }
}

#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        TIMER
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_event(timer::Event::Update);
    })
}
