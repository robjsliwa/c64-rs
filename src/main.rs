use crate::emuc64::{CPU, Memory};

mod emuc64;

fn main() {
    let mut mem = Memory::new();
    let mut cpu = CPU::new();

    loop {
        let mut input = String::new();
        println!("Enter command (step/load/display/quit):");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read command");
        match input.trim() {
            "step" => {
                cpu.step(&mut mem);
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

                mem.write_byte(address, value);
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
                    print!("{:#02X} ", mem.read_byte(start_address + i));
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
