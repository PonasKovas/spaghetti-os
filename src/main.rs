#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod game;

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_1_OFFSET + 8) });

#[no_mangle]
pub extern "C" fn _start() -> ! {
    set_timer_freq((1193182u32 / 20u32) as u16);

    IDT.load();

    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    // disable cursor
    unsafe {
        let mut p0 = Port::new(0x3D4);
        let mut p1 = Port::new(0x3D5);
        p0.write(0x0Au8);
        p1.write(0x20u8);
    }
    game::init_game();
    game::draw_border();

    loop {
        x86_64::instructions::hlt();
    }
}

fn set_timer_freq(divisor: u16) {
    // use channel 0
    let mut command_port = Port::new(0x43);
    unsafe {
        command_port.write(0b00110100u8);
    }

    // set the divisor
    let mut data_port = Port::new(0x40);
    unsafe {
        data_port.write(divisor as u8);
        data_port.write((divisor >> 8) as u8);
    }
}

fn say(text: &[u8]) {
    for (i, &byte) in text.iter().enumerate() {
        unsafe {
            *VGA_BUFFER.offset(i as isize * 2) = byte;
            *VGA_BUFFER.offset(i as isize * 2 + 1) = 0xf;
        }
    }
}

#[panic_handler]
fn _panic(_info: &PanicInfo) -> ! {
    say(b"panic.");
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    say(b"double fault occurred.");
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    game::tick();

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // control the snake with the keyboard input
    match scancode {
        30 | 75 => {
            // A letter or left arrow
            game::set_direction(game::Direction::Left);
        }
        31 | 80 => {
            // S letter or down arrow
            game::set_direction(game::Direction::Down);
        }
        32 | 77 => {
            // D letter or right arrow
            game::set_direction(game::Direction::Right);
        }
        17 | 72 => {
            // W letter or up arrow
            game::set_direction(game::Direction::Up);
        }
        28 => {
            // enter
            game::resume_game();
        }
        _ => {}
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

fn get_random_number() -> u32 {
    unsafe {
        let mut cport = Port::new(0x70);
        cport.write(0x00u8);
        let mut dport = Port::new(0x71);
        let r: u8 = dport.read();
        r as u32 * 1103515245 + 12345
    }
}
