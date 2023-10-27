use crate::cpu::Cpu;
use crate::memory::Memory;

mod cia1;
mod cpu;
mod memory;

fn main() {
    let mut mem = Memory::new();
    let mut cpu = Cpu::new(&mut mem);

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
        cpu.write_memory(i as u16, byte);
    }

    loop {
        let mut input = String::new();
        println!("Enter command (step/load/display/quit):");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read command");
        match input.trim() {
            "step" => {
                cpu.step();
                println!(
                    "Stepped. PC: {:#04X}, A: {:#02X}, X: {:#02X}, Y: {:#02X}",
                    cpu.pc, cpu.a, cpu.x, cpu.y
                );
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

                cpu.write_memory(address, value);
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
                    print!("{:#02X} ", cpu.read_memory(start_address + i));
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
