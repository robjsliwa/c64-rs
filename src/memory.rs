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

    pub fn new() -> Self {
        let mem_ram = vec![0; 65536]; // 64KB buffer initialized to zero
        let mem_rom = vec![0; 65536]; // 64KB buffer initialized to zero
        let banks = [0; 7]; // Initialize the banks array

        // TODO: Set up the default memory layout and load ROMs

        Memory {
            mem_ram,
            mem_rom,
            banks,
        }
    }

    // Writes a byte to RAM without performing I/O
    pub fn write_byte_no_io(&mut self, addr: u16, value: u8) {
        self.mem_ram[addr as usize] = value;
    }

    // Writes a byte to RAM handling I/O
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        // TODO: Implement logic for handling writes to special memory addresses
        // Placeholder for VIC, CIA1, CIA2 interactions
        self.mem_ram[addr as usize] = value;
    }

    // Reads a byte from RAM or ROM depending on the bank configuration
    pub fn read_byte(&self, addr: u16) -> u8 {
        // TODO: Implement logic for handling reads from special memory addresses
        // Placeholder for VIC, CIA1, CIA2 interactions
        self.mem_ram[addr as usize]
    }

    // Reads a byte without performing I/O, always from RAM
    pub fn read_byte_no_io(&self, addr: u16) -> u8 {
        self.mem_ram[addr as usize]
    }

    // Sets up the memory bank configuration based on specific flags
    pub fn setup_memory_banks(&mut self, config: u8) {
        // Extract config bits
        let hiram = (config & 0x01) != 0; // Placeholder for kHIRAM
        let loram = (config & 0x02) != 0; // Placeholder for kLORAM
        let charen = (config & 0x04) != 0; // Placeholder for kCHAREN

        // TODO: Define constants for memory banks and bank configurations

        // Initialize everything to RAM
        for bank in &mut self.banks {
            *bank = 0; // Placeholder for kRAM
        }

        // Set ROM or IO based on the configuration
        // TODO: Update this based on actual memory mapping and bank configurations
        if hiram {
            self.banks[0] = 1; // Placeholder for kROM/Kernal
        }
        if loram && hiram {
            self.banks[1] = 1; // Placeholder for kROM/Basic
        }
        if charen && (loram || hiram) {
            self.banks[2] = 2; // Placeholder for kIO
        } else if charen && !loram && !hiram {
            self.banks[2] = 0; // Placeholder for kRAM
        } else {
            self.banks[2] = 1; // Placeholder for kROM
        }

        // TODO: Load ROMs and set other configurations as needed
    }

    // Mock implementation to load ROM data into memory
    pub fn load_rom(&mut self, rom_name: &str, address: u16) {
        // TODO: Implement actual ROM loading logic (e.g., from a file or other source)
        // For now, we'll just fill the memory at the given address with placeholder data

        let placeholder_data = vec![0xFF; 4096]; // 4KB of placeholder data
        for (i, &byte) in placeholder_data.iter().enumerate() {
            self.mem_rom[(address as usize) + i] = byte;
        }
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
}
