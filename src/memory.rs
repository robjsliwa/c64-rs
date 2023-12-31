use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

// DRAM
// $0000-$00FF  Page 0        Zeropage addressing
// $0100-$01FF  Page 1        Enhanced Zeropage contains the stack
// $0200-$02FF  Page 2        Operating System and BASIC pointers
// $0300-$03FF  Page 3        Operating System and BASIC pointers
// $0400-$07FF  Page 4-7      Screen Memory
// $0800-$9FFF  Page 8-159    Free BASIC program storage area (38911 bytes)
// $A000-$BFFF  Page 160-191  Free machine language program storage area (when switched-out with ROM)
// $C000-$CFFF  Page 192-207  Free machine language program storage area
// $D000-$D3FF  Page 208-211
// $D400-$D4FF  Page 212-215
// $D800-$DBFF  Page 216-219
// $DC00-$DCFF  Page 220
// $DD00-$DDFF  Page 221
// $DE00-$DFFF  Page 222-223  Reserved for interface extensions
// $E000-$FFFF  Page 224-255  Free machine language program storage area (when switched-out with ROM)

enum BankCfg {
    Rom = 0,
    Ram = 1,
    Io = 2,
}

impl BankCfg {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn from_u8(value: u8) -> Option<BankCfg> {
        match value {
            0 => Some(BankCfg::Rom),
            1 => Some(BankCfg::Ram),
            2 => Some(BankCfg::Io),
            _ => None,
        }
    }
}

enum Banks {
    BankBasic = 3,
    BankCharen = 5,
    BankKernal = 6,
}

impl Banks {
    fn to_usize(self) -> usize {
        self as usize
    }
}

pub struct Memory {
    mem_ram: Vec<u8>, // RAM buffer
    mem_rom: Vec<u8>, // ROM buffer
    banks: [u8; 7],   // Memory bank configurations
                      // vic: Option<*mut Vic>, // Using raw pointers for external device references
                      // cia1: Option<*mut Cia1>,
                      // cia2: Option<*mut Cia2>,
                      // sid: Option<*mut Sid>,
}

impl Memory {
    pub const MEM_SIZE: usize = 0x10000;
    pub const BASE_ADDR_BASIC: u16 = 0xa000;
    pub const BASE_ADDR_KERNAL: u16 = 0xe000;
    pub const BASE_ADDR_STACK: u16 = 0x0100;
    pub const BASE_ADDR_SCREEN: u16 = 0x0400;
    pub const BASE_ADDR_CHARS: u16 = 0xd000;
    pub const BASE_ADDR_BITMAP: u16 = 0x0000;
    pub const BASE_ADDR_COLOR_RAM: u16 = 0xd800;
    pub const ADDR_RESET_VECTOR: u16 = 0xfffc;
    pub const ADDR_IRQ_VECTOR: u16 = 0xfffe;
    pub const ADDR_NMI_VECTOR: u16 = 0xfffa;
    pub const ADDR_DATA_DIRECTION: u16 = 0x0000;
    pub const ADDR_MEMORY_LAYOUT: u16 = 0x0001;
    pub const ADDR_COLOR_RAM: u16 = 0xd800;
    pub const ADDR_ZERO_PAGE: u16 = 0x0000;
    pub const ADDR_VIC_FIRST_PAGE: u16 = 0xd000;
    pub const ADDR_VIC_LAST_PAGE: u16 = 0xd300;
    pub const ADDR_CIA1_PAGE: u16 = 0xdc00;
    pub const ADDR_CIA2_PAGE: u16 = 0xdd00;
    pub const ADDR_BASIC_FIRST_PAGE: u16 = 0xa000;
    pub const ADDR_BASIC_LAST_PAGE: u16 = 0xbf00;
    pub const ADDR_KERNAL_FIRST_PAGE: u16 = 0xe000;
    pub const ADDR_KERNAL_LAST_PAGE: u16 = 0xff00;
    pub const LORAM: u8 = 1 << 0;
    pub const HIRAM: u8 = 1 << 1;
    pub const CHAREN: u8 = 1 << 2;

    pub fn new() -> Result<Self, String> {
        let mem_ram = vec![0; Memory::MEM_SIZE]; // 64KB buffer initialized to zero
        let mem_rom = vec![0; Memory::MEM_SIZE]; // 64KB buffer initialized to zero
        let banks = [0; 7]; // Initialize the banks array

        let mut memory = Memory {
            mem_ram,
            mem_rom,
            banks,
        };

        memory
            .setup_memory_banks(Self::LORAM | Self::HIRAM | Self::CHAREN)
            .map_err(|e| format!("Failed to load ROMs: {}", e))?;

        Ok(memory)
    }

    // Writes a byte to RAM without performing I/O
    pub fn write_byte_no_io(&mut self, addr: u16, value: u8) {
        self.mem_ram[addr as usize] = value;
    }

    // Writes a byte to RAM handling I/O
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let page = addr & 0xff00;

        if page == Self::ADDR_ZERO_PAGE {
            if addr == Self::ADDR_MEMORY_LAYOUT {
                self.setup_memory_banks(value)
                    .expect("Failed to set up memory banks");
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if page >= Self::ADDR_VIC_FIRST_PAGE && addr <= Self::ADDR_VIC_LAST_PAGE {
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                // vic.write_register(addr&0x7f, value);
                todo!();
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if page == Self::ADDR_CIA1_PAGE {
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                // cia1.write_register(addr & 0x0f, value);
                todo!();
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if page == Self::ADDR_CIA2_PAGE {
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                // cia2.write_register(addr&0x0f, value);
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else {
            self.mem_ram[addr as usize] = value;
        }
    }

    // Reads a byte from RAM or ROM depending on the bank configuration
    pub fn read_byte(&self, addr: u16) -> u8 {
        let page = addr & 0xff00;
        match page {
            _ if (Self::ADDR_VIC_FIRST_PAGE..=Self::ADDR_VIC_LAST_PAGE).contains(&page) => {
                // match self.banks[Banks::BankCharen.to_usize()] {
                //     BankCfg::Io => self.vic.read_register(addr & 0x7f),
                //     BankCfg::Rom => self.mem_rom[addr as usize],
                //     _ => self.mem_ram[addr as usize],
                // }
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    // self.vic.read_register(addr & 0x7f)
                    todo!();
                } else if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Rom.as_u8() {
                    self.mem_rom[addr as usize]
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if page == Self::ADDR_CIA1_PAGE => {
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    // self.cia1.read_register(addr & 0x0f)
                    todo!();
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if page == Self::ADDR_CIA2_PAGE => {
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    // self.cia2.read_register(addr & 0x0f)
                    todo!();
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if (Self::ADDR_BASIC_FIRST_PAGE..=Self::ADDR_BASIC_LAST_PAGE).contains(&page) => {
                if self.banks[Banks::BankBasic.to_usize()] == BankCfg::Rom.as_u8() {
                    self.mem_rom[addr as usize]
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if (Self::ADDR_KERNAL_FIRST_PAGE..=Self::ADDR_KERNAL_LAST_PAGE).contains(&page) => {
                if self.banks[Banks::BankKernal.to_usize()] == BankCfg::Rom.as_u8() {
                    self.mem_rom[addr as usize]
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ => self.mem_ram[addr as usize],
        }
    }

    // Reads a byte without performing I/O, always from RAM
    pub fn read_byte_no_io(&self, addr: u16) -> u8 {
        self.mem_ram[addr as usize]
    }

    // Sets up the memory bank configuration based on specific flags
    pub fn setup_memory_banks(&mut self, config: u8) -> io::Result<()> {
        let hiram = (config & Self::HIRAM) != 0;
        let loram = (config & Self::LORAM) != 0;
        let charen = (config & Self::CHAREN) != 0;

        // Initialize everything to RAM
        for bank in self.banks.iter_mut() {
            *bank = BankCfg::Ram.as_u8();
        }

        self.load_rom("basic.901226-01.bin", Self::BASE_ADDR_BASIC)?;
        self.load_rom("characters.901225-01.bin", Self::BASE_ADDR_CHARS)?;
        self.load_rom("kernal.901227-03.bin", Self::BASE_ADDR_KERNAL)?;

        // Set banks based on configuration
        if hiram {
            self.banks[Banks::BankKernal.to_usize()] = BankCfg::Rom.as_u8();
        }
        if loram && hiram {
            self.banks[Banks::BankBasic.to_usize()] = BankCfg::Rom.as_u8();
        }
        if charen && (loram || hiram) {
            self.banks[Banks::BankCharen.to_usize()] = BankCfg::Io.as_u8();
        } else if charen && !loram && !hiram {
            self.banks[Banks::BankCharen.to_usize()] = BankCfg::Ram.as_u8();
        } else {
            self.banks[Banks::BankCharen.to_usize()] = BankCfg::Rom.as_u8();
        }

        // Write the configuration to the zero page
        // Adjust this part according to your implementation of write_byte_no_io
        self.write_byte_no_io(Self::ADDR_MEMORY_LAYOUT, config);

        Ok(())
    }

    /// Reads a 16-bit word from memory at the given address
    pub fn read_word(&self, addr: u16) -> u16 {
        let lsb = self.read_byte(addr) as u16;
        let msb = self.read_byte(addr + 1) as u16;
        (msb << 8) | lsb
    }

    /// Writes a 16-bit word to memory at the given address
    pub fn write_word(&mut self, addr: u16, value: u16) {
        let lsb = (value & 0xFF) as u8;
        let msb = ((value >> 8) & 0xFF) as u8;
        self.write_byte(addr, lsb);
        self.write_byte(addr + 1, msb);
    }

    pub fn load_rom(&mut self, filename: &str, baseaddr: u16) -> io::Result<()> {
        let path = Path::new("./assets/roms/").join(filename);
        let mut file = File::open(path)?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let baseaddr = baseaddr as usize;
        for (i, &byte) in contents.iter().enumerate() {
            if let Some(slot) = self.mem_rom.get_mut(baseaddr + i) {
                *slot = byte;
            } else {
                break; // Prevent writing beyond the buffer's end
            }
        }

        Ok(())
    }

    pub fn load_ram(&mut self, filename: &str, baseaddr: u16) -> io::Result<()> {
        let path = Path::new("./assets/").join(filename);
        let mut file = File::open(path)?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let baseaddr = baseaddr as usize;
        for (i, &byte) in contents.iter().enumerate() {
            if let Some(slot) = self.mem_ram.get_mut(baseaddr + i) {
                *slot = byte;
            } else {
                break; // Prevent writing beyond the buffer's end
            }
        }

        Ok(())
    }
}
