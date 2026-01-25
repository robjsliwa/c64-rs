use crate::cia1::Cia1;
use crate::cia2::Cia2;
use crate::sid::Sid;
use crate::vic::Vic;
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::rc::Weak;

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

    #[allow(dead_code)] // Helper for bank configuration
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
    mem_ram: Vec<u8>,         // RAM buffer
    mem_rom: Vec<u8>,         // ROM buffer
    banks: [u8; 7],           // Memory bank configurations
    color_ram: [u8; 1024],    // Color RAM (1KB for $D800-$DBFF)
    cia1: Option<Weak<RefCell<Cia1>>>,
    cia2: Option<Weak<RefCell<Cia2>>>,
    vic: Option<Weak<RefCell<Vic>>>,
    sid: Option<Weak<RefCell<Sid>>>,
}

impl Memory {
    pub const MEM_SIZE: usize = 0x10000;
    pub const BASE_ADDR_BASIC: u16 = 0xa000;
    pub const BASE_ADDR_KERNAL: u16 = 0xe000;
    pub const BASE_ADDR_STACK: u16 = 0x0100;
    #[allow(dead_code)] // Memory layout constants for future use
    pub const BASE_ADDR_SCREEN: u16 = 0x0400;
    #[allow(dead_code)]
    pub const BASE_ADDR_CHARS: u16 = 0xd000;
    #[allow(dead_code)]
    pub const BASE_ADDR_BITMAP: u16 = 0x0000;
    #[allow(dead_code)]
    pub const BASE_ADDR_COLOR_RAM: u16 = 0xd800;
    #[allow(dead_code)]
    pub const ADDR_RESET_VECTOR: u16 = 0xfffc;
    pub const ADDR_IRQ_VECTOR: u16 = 0xfffe;
    pub const ADDR_NMI_VECTOR: u16 = 0xfffa;
    #[allow(dead_code)]
    pub const ADDR_DATA_DIRECTION: u16 = 0x0000;
    pub const ADDR_MEMORY_LAYOUT: u16 = 0x0001;
    #[allow(dead_code)]
    pub const ADDR_COLOR_RAM: u16 = 0xd800;
    pub const ADDR_ZERO_PAGE: u16 = 0x0000;
    pub const ADDR_VIC_FIRST_PAGE: u16 = 0xd000;
    pub const ADDR_VIC_LAST_PAGE: u16 = 0xd300;
    #[allow(dead_code)]
    pub const ADDR_COLOR_RAM_PAGE: u16 = 0xd800;
    pub const ADDR_CIA1_PAGE: u16 = 0xdc00;
    pub const ADDR_CIA2_PAGE: u16 = 0xdd00;
    pub const ADDR_SID_PAGE: u16 = 0xd400;
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
            color_ram: [0; 1024],
            cia1: None,
            cia2: None,
            vic: None,
            sid: None,
        };

        memory
            .setup_memory_banks(Self::LORAM | Self::HIRAM | Self::CHAREN)
            .map_err(|e| format!("Failed to load ROMs: {}", e))?;

        Ok(memory)
    }

    /// Register CIA1 for I/O routing
    pub fn register_cia1(&mut self, cia1: Weak<RefCell<Cia1>>) {
        self.cia1 = Some(cia1);
    }

    /// Register CIA2 for I/O routing
    pub fn register_cia2(&mut self, cia2: Weak<RefCell<Cia2>>) {
        self.cia2 = Some(cia2);
    }

    /// Register VIC for I/O routing
    pub fn register_vic(&mut self, vic: Weak<RefCell<Vic>>) {
        self.vic = Some(vic);
    }

    /// Register SID for I/O routing
    pub fn register_sid(&mut self, sid: Weak<RefCell<Sid>>) {
        self.sid = Some(sid);
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
        } else if page >= Self::ADDR_VIC_FIRST_PAGE && page <= Self::ADDR_VIC_LAST_PAGE {
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                if let Some(ref vic_weak) = self.vic {
                    if let Some(vic) = vic_weak.upgrade() {
                        vic.borrow_mut().write_register((addr & 0x3F) as u8, value);
                        return;
                    }
                }
                // Fallback to RAM if VIC not registered
                self.mem_ram[addr as usize] = value;
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if addr >= 0xD800 && addr <= 0xDBFF {
            // Color RAM ($D800-$DBFF)
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                // Color RAM is only 4 bits wide
                self.color_ram[(addr - 0xD800) as usize] = value & 0x0F;
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if page == Self::ADDR_SID_PAGE {
            // SID registers ($D400-$D4FF)
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                if let Some(ref sid_weak) = self.sid {
                    if let Some(sid) = sid_weak.upgrade() {
                        sid.borrow_mut().write((addr & 0x1F) as u8, value);
                        return;
                    }
                }
                // Fallback to RAM if SID not registered
                self.mem_ram[addr as usize] = value;
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if page == Self::ADDR_CIA1_PAGE {
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                if let Some(ref cia1_weak) = self.cia1 {
                    if let Some(cia1) = cia1_weak.upgrade() {
                        cia1.borrow_mut().write_register((addr & 0x0F) as u8, value);
                        return;
                    }
                }
                // Fallback to RAM if CIA1 not registered
                self.mem_ram[addr as usize] = value;
            } else {
                self.mem_ram[addr as usize] = value;
            }
        } else if page == Self::ADDR_CIA2_PAGE {
            if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                if let Some(ref cia2_weak) = self.cia2 {
                    if let Some(cia2) = cia2_weak.upgrade() {
                        cia2.borrow_mut().write_register((addr & 0x0F) as u8, value);
                        return;
                    }
                }
                // Fallback to RAM if CIA2 not registered
                self.mem_ram[addr as usize] = value;
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
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    if let Some(ref vic_weak) = self.vic {
                        if let Some(vic) = vic_weak.upgrade() {
                            return vic.borrow().read_register((addr & 0x3F) as u8);
                        }
                    }
                    // Return $FF if VIC not registered
                    0xFF
                } else if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Rom.as_u8() {
                    self.mem_rom[addr as usize]
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            // Color RAM ($D800-$DBFF)
            _ if addr >= 0xD800 && addr <= 0xDBFF => {
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    // Color RAM is only 4 bits wide, upper bits read as high
                    self.color_ram[(addr - 0xD800) as usize] | 0xF0
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if page == Self::ADDR_SID_PAGE => {
                // SID registers ($D400-$D4FF)
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    if let Some(ref sid_weak) = self.sid {
                        if let Some(sid) = sid_weak.upgrade() {
                            return sid.borrow().read((addr & 0x1F) as u8);
                        }
                    }
                    // Return $FF if SID not registered
                    0xFF
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if page == Self::ADDR_CIA1_PAGE => {
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    if let Some(ref cia1_weak) = self.cia1 {
                        if let Some(cia1) = cia1_weak.upgrade() {
                            return cia1.borrow_mut().read_register((addr & 0x0F) as u8);
                        }
                    }
                    // Return $FF if CIA1 not registered
                    0xFF
                } else {
                    self.mem_ram[addr as usize]
                }
            }
            _ if page == Self::ADDR_CIA2_PAGE => {
                if self.banks[Banks::BankCharen.to_usize()] == BankCfg::Io.as_u8() {
                    if let Some(ref cia2_weak) = self.cia2 {
                        if let Some(cia2) = cia2_weak.upgrade() {
                            return cia2.borrow_mut().read_register((addr & 0x0F) as u8);
                        }
                    }
                    // Return $FF if CIA2 not registered
                    0xFF
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
    #[allow(dead_code)] // Utility for direct RAM access, used in debugging
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
    #[allow(dead_code)] // Utility method for 16-bit writes
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

    /// Read memory as VIC sees it (with banking)
    ///
    /// Bank selection (CIA2 PRA bits 0-1, inverted):
    /// - %00 = Bank 3: $C000-$FFFF
    /// - %01 = Bank 2: $8000-$BFFF
    /// - %10 = Bank 1: $4000-$7FFF
    /// - %11 = Bank 0: $0000-$3FFF
    pub fn vic_read_byte(&self, vic_addr: u16, bank: u8) -> u8 {
        let base: u16 = match bank & 0x03 {
            0 => 0xC000,
            1 => 0x8000,
            2 => 0x4000,
            _ => 0x0000,
        };

        let address = base.wrapping_add(vic_addr & 0x3FFF);

        // Character ROM visible in actual VIC banks 0 and 2 at $1000-$1FFF
        // With inverted bank selection: bank=3 is actual bank 0, bank=1 is actual bank 2
        if bank == 3 || bank == 1 {
            let local_addr = vic_addr & 0x3FFF;
            if (0x1000..=0x1FFF).contains(&local_addr) {
                return self.mem_rom[(Self::BASE_ADDR_CHARS + (local_addr & 0x0FFF)) as usize];
            }
        }

        self.mem_ram[address as usize]
    }

    /// Dump memory contents for debugging
    #[allow(dead_code)]
    pub fn dump(&self, start: u16, length: u16) -> String {
        let mut output = String::new();
        let end = start.saturating_add(length);
        for addr in (start..end).step_by(16) {
            output.push_str(&format!("${:04X}: ", addr));
            for offset in 0..16u16 {
                if addr.wrapping_add(offset) < end {
                    output.push_str(&format!(
                        "{:02X} ",
                        self.mem_ram[addr.wrapping_add(offset) as usize]
                    ));
                }
            }
            output.push('\n');
        }
        output
    }
}
