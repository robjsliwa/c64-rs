use crate::cia1::Cia1;
use crate::cia2::Cia2;
use crate::cpu::Cpu;
use crate::io::IO;
use crate::memory::Memory;
use clap::{command, Command};
use std::cell::RefCell;
use std::rc::Rc;

mod cia1;
mod cia2;
mod common;
mod cpu;
mod io;
mod memory;

fn debug(cpu: Rc<RefCell<Cpu>>, cia1: Rc<RefCell<Cia1>>) {
    // TEMP: Load the machine code into memory (for our sample program)
    // LDX #$03      ; Load X register with the number 3
    // LDA #$05      ; Load accumulator with the number 5
    // ADC #$00      ; Add 0 to accumulator (effectively adding X because of the previously set carry)
    // STA $0200     ; Store accumulator (result) at memory location $0200
    // JMP $0004     ; Jump to this instruction (creates an infinite loop)

    let program: [u8; 12] = [
        0xA2, 0x03, 0xA9, 0x05, 0x69, 0x00, 0x8D, 0x00, 0x02, 0x4C, 0x04, 0x00,
    ];
    for (i, &byte) in program.iter().enumerate() {
        cpu.borrow_mut().write_memory(i as u16, byte);
    }

    loop {
        let mut input = String::new();
        println!("Enter command (step/load/display/quit):");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read command");
        match input.trim() {
            "step" => {
                cpu.borrow_mut().step();
                println!(
                    "Stepped. PC: {:#04X}, A: {:#02X}, X: {:#02X}, Y: {:#02X}",
                    cpu.borrow().pc,
                    cpu.borrow().a,
                    cpu.borrow().x,
                    cpu.borrow().y
                );
                cia1.borrow_mut().step();
            }
            "load" => {
                println!("Enter memory address (hex):");
                let mut address_input = String::new();
                std::io::stdin()
                    .read_line(&mut address_input)
                    .expect("Failed to read address");
                let address =
                    u16::from_str_radix(address_input.trim(), 16).expect("Failed to parse address");

                println!("Enter byte value (hex):");
                let mut value_input = String::new();
                std::io::stdin()
                    .read_line(&mut value_input)
                    .expect("Failed to read value");
                let value =
                    u8::from_str_radix(value_input.trim(), 16).expect("Failed to parse value");

                cpu.borrow_mut().write_memory(address, value);
                println!("Loaded {:#02X} into {:#04X}", value, address);
            }
            "display" => {
                println!("Enter start memory address (hex):");
                let mut address_input = String::new();
                std::io::stdin()
                    .read_line(&mut address_input)
                    .expect("Failed to read address");
                let start_address =
                    u16::from_str_radix(address_input.trim(), 16).expect("Failed to parse address");

                for i in 0..0x10 {
                    print!("{:#02X} ", cpu.borrow().read_memory(start_address + i));
                }
                println!();
            }
            "quit" => {
                println!("Exiting emulator.");
                break;
            }
            _ => {
                println!("Unknown command. Please enter a valid command.");
            }
        }
    }
}

// uns Klaus Dormann's 6502 test suite
//
// https://github.com/Klaus2m5/6502_65C02_functional_tests
fn test_cpu(cpu: Rc<RefCell<Cpu>>) {
    let mut pc: u16 = 0x0;
    cpu.borrow_mut()
        .memory
        .write_byte(Memory::ADDR_MEMORY_LAYOUT, 0);
    cpu.borrow_mut()
        .memory
        .load_ram("tests/6502_functional_test.bin", 0x400)
        .unwrap();
    cpu.borrow_mut().pc = 0x400;
    loop {
        println!("PC: {:#04X}", pc);
        if pc == cpu.borrow().pc {
            println!("Infinit loop at {:#04X}", pc);
            break;
        } else if cpu.borrow().pc == 0x3463 {
            println!("Passed!");
            break;
        }
        pc = cpu.borrow().pc;
        if !cpu.borrow_mut().step() {
            break;
        }
    }
}

fn run_c64(
    cpu: Rc<RefCell<Cpu>>,
    cia1: Rc<RefCell<Cia1>>,
    cia2: Rc<RefCell<Cia2>>,
    io: Rc<RefCell<IO>>,
) {
    loop {
        if !cia1.borrow_mut().step() {
            break;
        }
        if !cpu.borrow_mut().step() {
            break;
        }

        if !io.borrow_mut().step() {
            break;
        }
    }
}

fn main() -> Result<(), String> {
    let mut mem = Memory::new()?;
    let cpu = Rc::new(RefCell::new(Cpu::new(&mut mem)));
    let io = Rc::new(RefCell::new(IO::new(cpu.clone())?));
    let cia1 = Rc::new(RefCell::new(Cia1::new(cpu.clone(), io.clone())));
    let cia2 = Rc::new(RefCell::new(Cia2::new(cpu.clone())));

    let matches = command!()
        .subcommand(Command::new("debug"))
        .subcommand(Command::new("test"))
        .get_matches();

    match matches.subcommand_name() {
        Some("debug") => {
            println!("Debug mode enabled");
            debug(cpu, cia1);
            return Ok(());
        }
        Some("test") => {
            println!("Test mode enabled");
            test_cpu(cpu);
            return Ok(());
        }
        _ => run_c64(cpu, cia1, cia2, io),
    }

    Ok(())
}
