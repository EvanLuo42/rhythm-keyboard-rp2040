#![no_std]
#![no_main]

use defmt_rtt as _;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use usb_device::{class_prelude::UsbBusAllocator, prelude::{UsbDeviceBuilder, UsbVidPid}};
use usbd_hid::{hid_class::HIDClass, descriptor::{KeyboardReport, SerializedDescriptor}};

const USB_HOST_POLL_MS: u8 = 10;

const XTAL_FREQ_HZ: u32= 12_000_000u32;

const KEY_D: u8 = 0x07;
const KEY_F: u8 = 0x09;
const KEY_J: u8 = 0x0d;
const KEY_K: u8 = 0x0e;

#[bsp::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let usb_bus = UsbBusAllocator::new(bsp::hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let row0 = pins.gpio26.into_pull_up_input();
    let row1 = pins.gpio27.into_pull_up_input();
    let mut col0 = pins.gpio0.into_push_pull_output();
    let mut col1 = pins.gpio1.into_push_pull_output();
    
    let mut led = pins.led.into_push_pull_output();

    let mut usb_hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), USB_HOST_POLL_MS);
    
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27da))
        .manufacturer("EvanLuo42")
        .product("Rhythm Keyboard RP2040")
        .serial_number("0")
        .device_class(0)
        .build();

    led.set_high().unwrap();
    delay.delay_ms(500);
    led.set_low().unwrap();
    
    loop {
        usb_dev.poll(&mut [&mut usb_hid]);

        col0.set_high().unwrap();

        if row0.is_high().unwrap() {
            send_key_press(&usb_hid, &mut delay, KEY_D);
        }
        
        if row1.is_high().unwrap() {
            send_key_press(&usb_hid, &mut delay, KEY_F)
        }

        col0.set_low().unwrap();
        col1.set_high().unwrap();

        if row0.is_high().unwrap() {
            send_key_press(&usb_hid, &mut delay, KEY_J);
        }
        
        if row1.is_high().unwrap() {
            send_key_press(&usb_hid, &mut delay, KEY_K)
        }

        col1.set_low().unwrap();
    }
}

fn send_key_press(
    usb_hid: &HIDClass<bsp::hal::usb::UsbBus>,
    delay: &mut cortex_m::delay::Delay,
    key_code: u8,
) {
    let mut keyboard_report = KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0; 6],
    };
    keyboard_report.keycodes[0] = key_code;
    usb_hid.push_input(&keyboard_report).unwrap();
    delay.delay_ms(USB_HOST_POLL_MS.into());

    keyboard_report.keycodes[0] = 0;
    usb_hid.push_input(&keyboard_report).unwrap();
    delay.delay_ms(USB_HOST_POLL_MS.into());
}