use crate::cia1::Cia1;
use crate::cia2::Cia2;
use crate::cpu::Cpu;
use crate::io::IO;
use crate::loader::Loader;
use crate::memory::Memory;
use crate::sid::Sid;
use crate::vic::Vic;
use clap::{command, Command};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

mod audio;
mod cia;
mod cia1;
mod cia2;
mod cpu;
mod io;
mod loader;
mod memory;
mod sid;
mod vic;

fn debug(cpu: Rc<RefCell<Cpu>>, cia1: Rc<RefCell<Cia1>>, _memory: Rc<RefCell<Memory>>) {
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

// Runs Klaus Dormann's 6502 test suite
//
// https://github.com/Klaus2m5/6502_65C02_functional_tests
fn test_cpu(cpu: Rc<RefCell<Cpu>>, memory: Rc<RefCell<Memory>>) {
    let mut last_pc: u16 = 0x0;
    memory
        .borrow_mut()
        .write_byte(Memory::ADDR_MEMORY_LAYOUT, 0);
    memory
        .borrow_mut()
        .load_ram("tests/6502_functional_test.bin", 0x400)
        .unwrap();
    cpu.borrow_mut().pc = 0x400;

    let mut cycles: u64 = 0;
    let max_cycles: u64 = 100_000_000;

    loop {
        let pc_before = cpu.borrow().pc;

        if !cpu.borrow_mut().step() {
            println!("Unknown opcode encountered at ${:04X}", pc_before);
            break;
        }

        let current_pc = cpu.borrow().pc;

        // Success: test complete at $3469
        if current_pc == 0x3469 {
            println!("Test PASSED after {} cycles!", cycles);
            break;
        }

        // Trap detection: PC stuck in infinite loop
        if current_pc == last_pc {
            // Get test number from memory for debugging
            let test_num = memory.borrow().read_byte(0x0200);
            println!(
                "Test FAILED: trapped at ${:04X}, test number: ${:02X}",
                current_pc, test_num
            );
            break;
        }

        last_pc = current_pc;

        cycles += 1;
        if cycles >= max_cycles {
            println!(
                "Test timeout after {} cycles at ${:04X}",
                cycles,
                cpu.borrow().pc
            );
            break;
        }
    }
}

fn run_c64(
    cpu: Rc<RefCell<Cpu>>,
    cia1: Rc<RefCell<Cia1>>,
    cia2: Rc<RefCell<Cia2>>,
    vic: Rc<RefCell<Vic>>,
    sid: Rc<RefCell<Sid>>,
    io: Rc<RefCell<IO>>,
    audio_state: Arc<Mutex<audio::SidState>>,
    mut loader: Option<Loader>,
) {
    loop {
        if !cia1.borrow_mut().step() {
            break;
        }
        if !cia2.borrow_mut().step() {
            break;
        }
        if !vic.borrow_mut().step() {
            break;
        }

        // Clock SID chip
        {
            let mut sid_mut = sid.borrow_mut();
            sid_mut.clock();

            // Generate audio sample if ready
            if sid_mut.should_generate_sample() {
                let sample = sid_mut.generate_sample();

                // Push sample to audio buffer
                if let Ok(mut state) = audio_state.lock() {
                    state.push_sample(sample);
                    state.clear_old_samples();
                }
            }
        }

        // Step loader if present
        if let Some(ref mut loader) = loader {
            loader.emulate();
        }

        if !cpu.borrow_mut().step() {
            break;
        }

        io.borrow_mut().process_events();
        if !io.borrow().step() {
            break;
        }
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    // Initialize audio
    let (_audio_device, audio_state) = audio::init_audio(&sdl_context)?;

    let memory = Rc::new(RefCell::new(Memory::new()?));
    let cpu = Rc::new(RefCell::new(Cpu::new(memory.clone())));
    let io = Rc::new(RefCell::new(IO::new(cpu.clone(), &sdl_context)?));
    let cia1 = Rc::new(RefCell::new(Cia1::new(
        Rc::downgrade(&cpu),
        Rc::downgrade(&io),
    )));
    let cia2 = Rc::new(RefCell::new(Cia2::new(Rc::downgrade(&cpu))));
    let vic = Rc::new(RefCell::new(Vic::new(
        Rc::downgrade(&memory),
        Rc::downgrade(&cpu),
        io.clone(),
    )));
    let sid = Rc::new(RefCell::new(Sid::new(44100))); // 44.1kHz sample rate

    // Register devices with memory for I/O routing
    {
        let mut mem = memory.borrow_mut();
        mem.register_cia1(Rc::downgrade(&cia1));
        mem.register_cia2(Rc::downgrade(&cia2));
        mem.register_vic(Rc::downgrade(&vic));
        mem.register_sid(Rc::downgrade(&sid));
    }

    // Reset CPU to initialize PC from reset vector
    cpu.borrow_mut().reset();

    let matches = command!()
        .subcommand(Command::new("debug"))
        .subcommand(Command::new("test"))
        .arg(
            clap::Arg::new("program")
                .help("Program file to load (.prg or .bas)")
                .value_name("FILE")
                .index(1),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("debug") => {
            println!("Debug mode enabled");
            debug(cpu, cia1, memory);
            return Ok(());
        }
        Some("test") => {
            println!("Test mode enabled");
            test_cpu(cpu, memory);
            return Ok(());
        }
        _ => {
            // Check for program argument
            let loader = if let Some(program_path) = matches.get_one::<String>("program") {
                match Loader::load_file(program_path) {
                    Ok(mut loader) => {
                        println!("Loading program: {}", program_path);
                        loader.set_references(
                            Rc::downgrade(&memory),
                            Rc::downgrade(&cpu),
                            Rc::downgrade(&io),
                        );
                        Some(loader)
                    }
                    Err(e) => {
                        eprintln!("Error loading program: {:?}", e);
                        None
                    }
                }
            } else {
                None
            };

            run_c64(cpu, cia1, cia2, vic, sid, io, audio_state, loader)
        }
    }

    Ok(())
}
